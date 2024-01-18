use std::env;

use crate::{
    db_executor::MyDBQueryExecutor,
    domain::{Admin, AdminId, ContactId, EndpointData, EndpointId, OutageId},
    notification_sender::{
        create_notification_sender_and_receiver, TelegramNotificationSender, TelegramReceiver,
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
pub trait NotificationSender: Send + Sync {
    async fn send_notification(&self, x: NotificationData);
}

#[async_trait::async_trait]
pub trait ResponseConsumer: Send + Sync {
    // If still is down -
    async fn consume_response(&mut self, response: ResponseData);
}

// pub trait ResponseListener: Send + Sync {
//     async fn listen_for_responses() {

//     }
// }

#[derive(Clone, Debug)]
pub struct NotificationData {
    pub admin: AdminId,
    pub outage_id: OutageId,
    pub endpoint: EndpointId,
    pub contact_id: ContactId,
    pub is_first: bool,
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
    ntf_sender: TelegramNotificationSender,
}

impl NotificationService {
    pub fn new(
        db_executor: MyDBQueryExecutor,
        ntf_sender: TelegramNotificationSender,
    ) -> NotificationService {
        NotificationService {
            db_executor,
            ntf_sender,
        }
    }

    pub async fn init_service(
        &self,
        ntf_receiver: TelegramReceiver,
        response_receiver: Receiver<ResponseData>,
    ) {
        self.spawn_receiver_task(response_receiver).await;
        self.spawn_notification_receiver_task(ntf_receiver).await;
        self.run_main_loop().await;
    }

    async fn run_main_loop(&self) {
        loop {
            let x = self.db_executor.get_endpoints_to_process().await;
            if let Err(error) = x {
                log::error!("Errror getting endpoints to process: {:?}", error);
            } else {
                let v = x.unwrap();
                log::debug!("Got {:?} dead endpoints!", v.len());
                let db_executor = self.db_executor.clone();
                let sender = self.ntf_sender.clone();

                let futures: FuturesUnordered<_> = v
                    .into_iter()
                    .map(|x| {
                        let db_executor = db_executor.clone();
                        let sender = sender.clone();
                        tokio::spawn(async move {
                            let ntf_data =
                                Self::get_notification_from_endpoint_data(&db_executor, x).await;
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
    //     pub contact_id: ContactId,
    //     pub is_first: bool,
    // }

    async fn get_notification_from_endpoint_data(
        db_executor: &MyDBQueryExecutor,
        endpoint_data: EndpointData,
    ) -> NotificationData {
        let is_first = !endpoint_data.ntf_first_responded;
        let admin = if is_first {
            endpoint_data.ntf_primary_admin
        } else {
            endpoint_data.ntf_secondary_admin
        };

        let admin_data = db_executor.get_admin_data(admin.clone()).await.unwrap();

        NotificationData {
            admin: admin_data.admin_id,
            outage_id: endpoint_data.outage_id.unwrap(),
            endpoint: endpoint_data.endpoint_id,
            contact_id: admin_data.contact_id,
            is_first: is_first,
        }
    }

    async fn send_notification_and_mark_it(
        db_executor: &MyDBQueryExecutor,
        ntf_sender: &TelegramNotificationSender,
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

    async fn spawn_receiver_task(&self, mut response_receiver: Receiver<ResponseData>) {
        let db_executor = self.db_executor.clone();
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
                            "marked endpoint: {}, outage: {:?} ",
                            response_data.endpoint,
                            response_data.outage_id
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
    async fn spawn_notification_receiver_task(&self, ntf_receiver: TelegramReceiver) {
        tokio::spawn(async move {
            ntf_receiver.listen_for_replies().await;
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
    let (ntf_sender, ntf_receiver) = create_notification_sender_and_receiver(sender);
    let ntf_service: NotificationService = NotificationService::new(db_executor, ntf_sender);
    ntf_service.init_service(ntf_receiver, receiver).await;
}

pub mod constants {
    pub const RESPONSE_DATA_CHANNEL_BUFFER_SIZE: usize = 128;
}
