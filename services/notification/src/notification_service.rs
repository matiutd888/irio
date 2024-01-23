#![allow(dead_code)]

use std::env;

use crate::{
    db_executor::MyDBQueryExecutor,
    domain::{Admin, AdminId, ContactId, EndpointData, EndpointId, OutageId},
    notification_sender::{
        create_telegram_notification_sender_and_receiver, EmailNotificationSender,
        TelegramNotificationResponseListener, TelegramNotificationSender,
    },
};
use ::futures::stream::FuturesUnordered;
use anyhow::Result;
use tokio::sync::mpsc::{channel, Receiver};
use uuid::Uuid;

#[async_trait::async_trait]
pub trait DBQueryExecutor: Send + Sync {
    /*
        Read all endpoints that
        1. Are down
        2. Either
        2a) are not handled (ntf_is_handled is false)
        2b) are handled but they are being too slow (ntf_is_being_handled_timestamp is too old).
        3. Either
        3a) is notification sent is false
        3b) is notification sent is true BUT ntf_first_notification_send_time is too old

        Run LWT to update all the nodes if they are not handled already and are not down. Say that they are handled.
    */
    async fn get_endpoints_to_process(&self) -> Result<Vec<EndpointData>>;
    async fn mark_first_notification_sent(
        &self,
        endpoint_id: EndpointId,
        outage_id: OutageId,
    ) -> Result<()>;
    async fn mark_second_notification_sent(
        &self,
        endpoint_id: EndpointId,
        outage_id: OutageId,
    ) -> Result<()>;
    async fn mark_endpoint_responded(
        &self,
        endpoint_id: EndpointId,
        outage_id: OutageId,
    ) -> Result<()>;

    async fn get_admin_data(&self, admin_id: AdminId) -> Result<Admin>;
}

// Send notification to given
#[async_trait::async_trait]
pub trait NotificationSender: Send + Sync + Clone {
    async fn send_notification(&self, x: NotificationData);
}

// #[async_trait::async_trait]
// pub trait ResponseConsumer: Send + Sync {
//     // If still is down -
//     async fn consume_response(&mut self, response: ResponseData);
// }

#[derive(Clone)]
enum ImplementedNotificationSender {
    Telegram(TelegramNotificationSender),
    Email(EmailNotificationSender),
}

#[async_trait::async_trait]
impl NotificationSender for ImplementedNotificationSender {
    async fn send_notification(&self, x: NotificationData) {
        match &self {
            ImplementedNotificationSender::Telegram(s) => s.send_notification(x).await,
            ImplementedNotificationSender::Email(s) => s.send_notification(x).await,
        }
    }
}

#[derive(Clone)]
pub struct AggregatedNotificationSender {
    senders: Vec<ImplementedNotificationSender>,
}

impl AggregatedNotificationSender {
    fn create(
        t_sender: TelegramNotificationSender,
        _email_sender: Option<EmailNotificationSender>,
    ) -> AggregatedNotificationSender {
        let t = ImplementedNotificationSender::Telegram(t_sender);
        // let e: ImplementedNotificationSender = ImplementedNotificationSender::Email(email_sender);
        AggregatedNotificationSender { senders: vec![t] }
    }
}

enum ImplementedNotificationResponseListener {
    Telegram(TelegramNotificationResponseListener),
}

#[async_trait::async_trait]
impl ResponseListener for ImplementedNotificationResponseListener {
    async fn listen_for_responses(&self) {
        match &self {
            ImplementedNotificationResponseListener::Telegram(r) => r.listen_for_responses().await,
        }
    }
}

#[async_trait::async_trait]
impl NotificationSender for AggregatedNotificationSender {
    async fn send_notification(&self, x: NotificationData) {
        let futures = self
            .senders
            .clone()
            .into_iter()
            .map(|i| {
                let new_i = i.clone();
                let new_x = x.clone();
                tokio::spawn(async move {
                    new_i.send_notification(new_x).await;
                })
            })
            .collect::<FuturesUnordered<_>>();

        futures::future::join_all(futures).await;
    }
}

#[async_trait::async_trait]
pub trait ResponseListener: Send + Sync {
    async fn listen_for_responses(&self) {}
}

#[derive(Clone, Debug)]
pub struct NotificationData {
    pub admin: AdminId,
    pub outage_id: OutageId,
    pub endpoint: EndpointId,
    pub telegram_contact_id: ContactId,
    pub is_first: bool,
    pub http_address: String,
    pub email: String,
}

pub struct ResponseData {
    pub admin: AdminId,
    pub outage_id: OutageId,
    pub endpoint: EndpointId,
    pub is_first: bool,
}

pub struct ServiceParams {
    pub endpoints_in_query: u32,
    pub secs_wait_when_handled: u32,
    pub service_uuid: Uuid,
}

pub fn init_service_params() -> ServiceParams {
    let n_endpoints_in_query = env::var("ENDPOINTS_IN_QUERY")
        .ok()
        .and_then(|x| x.parse::<u32>().ok())
        .unwrap_or(1);
    let secs_wait_when_handled = env::var("SECS_WAIT_WHEN_HANDLED")
        .ok()
        .and_then(|x| x.parse::<u32>().ok())
        .unwrap_or(40);

    let id = Uuid::new_v4();
    ServiceParams {
        endpoints_in_query: n_endpoints_in_query,
        secs_wait_when_handled: secs_wait_when_handled,
        service_uuid: id,
    }
}

struct NotificationService {
    db_executor: MyDBQueryExecutor,
    ntf_sender: AggregatedNotificationSender,
}

impl NotificationService {
    pub fn new(
        db_executor: MyDBQueryExecutor,
        ntf_sender: AggregatedNotificationSender,
    ) -> NotificationService {
        NotificationService {
            db_executor,
            ntf_sender,
        }
    }

    pub async fn init_service(
        &self,
        telegram_ntf_receiver: TelegramNotificationResponseListener,
        response_data_receiver: Receiver<ResponseData>,
    ) {
        self.spawn_response_data_receiver_task(response_data_receiver)
            .await;
        self.spawn_notification_response_listener_task(
            ImplementedNotificationResponseListener::Telegram(telegram_ntf_receiver),
        )
        .await;
        Self::run_main_loop(self.db_executor.clone(), self.ntf_sender.clone()).await;
    }

    async fn run_main_loop(
        db_executor: MyDBQueryExecutor,
        ntf_sender: AggregatedNotificationSender,
    ) {
        loop {
            let x = db_executor.get_endpoints_to_process().await;
            if let Err(error) = x {
                log::error!("Errror getting endpoints to process: {:?}", error);
            } else {
                let v = x.unwrap();
                if v.len() > 0 {
                    log::info!("Got {:?} dead endpoints!", v.len());
                }
                let db_executor = db_executor.clone();
                let sender = ntf_sender.clone();

                let futures: FuturesUnordered<_> = v
                    .into_iter()
                    .map(|x| {
                        let db_executor = db_executor.clone();
                        let sender = sender.clone();
                        tokio::spawn(async move {
                            let ntf_data =
                                Self::get_notification_from_endpoint_data(&db_executor, x.clone())
                                    .await;
                            log::info!(
                                "Sending notification {:?} about endpoint {}",
                                ntf_data,
                                x.http_address
                            );
                            Self::send_notification_and_mark_it(&db_executor, &sender, ntf_data)
                                .await;
                        })
                    })
                    .collect::<FuturesUnordered<_>>();
                futures::future::join_all(futures).await;
            }
        }
    }

    // #[derive(Clone, Debug)]
    // pub struct NotificationData {
    //     pub admin: AdminId,
    //     pub outage_id: OutageId,
    //     pub endpoint: EndpointId,
    //     pub telegram_contact_id: ContactId,
    //     pub is_first: bool,
    // }

    async fn get_notification_from_endpoint_data(
        db_executor: &MyDBQueryExecutor,
        endpoint_data: EndpointData,
    ) -> NotificationData {
        let is_first = !endpoint_data.ntf_is_first_notification_sent;
        let admin = if is_first {
            endpoint_data.conf_primary_admin
        } else {
            endpoint_data.conf_secondary_admin
        };

        let admin_data = db_executor.get_admin_data(admin.clone()).await.unwrap();

        NotificationData {
            admin: admin_data.admin_id,
            outage_id: endpoint_data.outage_id.unwrap(),
            endpoint: endpoint_data.endpoint_id,
            telegram_contact_id: admin_data.telegram_contact_id,
            is_first: is_first,
            http_address: endpoint_data.http_address,
            email: admin_data.email_address,
        }
    }

    async fn send_notification_and_mark_it(
        db_executor: &MyDBQueryExecutor,
        ntf_sender: &AggregatedNotificationSender,
        ntf_data: NotificationData,
    ) {
        ntf_sender.send_notification(ntf_data.clone()).await;
        let result = if ntf_data.is_first {
            db_executor
                .mark_first_notification_sent(ntf_data.endpoint.clone(), ntf_data.outage_id)
                .await
        } else {
            db_executor
                .mark_second_notification_sent(ntf_data.endpoint.clone(), ntf_data.outage_id)
                .await
        };
        if let Err(error) = result {
            log::error!(
                "Error marking notification as sent {:?}: {:?}",
                ntf_data,
                error
            );
        } else {
            log::info!("Notification {:?} marked as sent succcessfully", ntf_data);
        }
    }

    async fn spawn_response_data_receiver_task(
        &self,
        mut response_receiver: Receiver<ResponseData>,
    ) {
        let db_executor: MyDBQueryExecutor = self.db_executor.clone();
        tokio::spawn(async move {
            loop {
                if let Some(response_data) = response_receiver.recv().await {
                    let x = db_executor
                        .mark_endpoint_responded(
                            response_data.endpoint.clone(),
                            response_data.outage_id,
                        )
                        .await;
                    if x.is_ok() {
                        log::info!(
                            "endpoint: {}, outage: {:?} marked as 'responded' by admin {}",
                            response_data.endpoint,
                            response_data.outage_id,
                            response_data.admin
                        );
                    } else {
                        log::error!(
                            "error marking endpoint: {}, outage {:?} as responded: {}",
                            response_data.endpoint,
                            response_data.outage_id,
                            x.err().unwrap()
                        );
                    }
                }
            }
        });
    }
    async fn spawn_notification_response_listener_task(
        &self,
        ntf_receiver: ImplementedNotificationResponseListener,
    ) {
        tokio::spawn(async move {
            ntf_receiver.listen_for_responses().await;
        });
    }
}

pub async fn run_notification_service() {
    let c = init_service_params();
    let db_executor = MyDBQueryExecutor::new(
        c.secs_wait_when_handled,
        c.endpoints_in_query,
        c.service_uuid,
    )
    .await;
    let (sender, receiver) = channel(constants::RESPONSE_DATA_CHANNEL_BUFFER_SIZE);
    let (telegram_ntf_sender, ntf_receiver) =
        create_telegram_notification_sender_and_receiver(sender);
    // let email_sender = EmailNotificationSender::new();
    let ntf_sender = AggregatedNotificationSender::create(telegram_ntf_sender, None);
    let ntf_service: NotificationService = NotificationService::new(db_executor, ntf_sender);
    ntf_service.init_service(ntf_receiver, receiver).await;
}

pub mod constants {
    pub const RESPONSE_DATA_CHANNEL_BUFFER_SIZE: usize = 128;
}
