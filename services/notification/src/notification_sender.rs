use teloxide::{
    dispatching::Dispatcher,
    payloads::SendMessageSetters,
    requests::{Request, Requester, ResponseResult},
    types::{Message, ParseMode, Update, UserId},
    Bot,
};

use teloxide::prelude::*;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::{
    domain::EndpointId,
    notification_service::{NotificationData, NotificationSender, ResponseData, ResponseListener},
};

#[derive(Debug, Clone)]
pub struct TelegramNotificationSender {
    bot: Bot,
}

#[derive(Debug, Clone)]
pub struct TelegramNotificationReceiver {
    bot: Bot,
    sender: Sender<ResponseData>,
}

impl TelegramNotificationSender {
    fn prepare_telegram_msg(msg: String) -> String {
        msg.replace("_", "\\_")
            .replace("*", "\\*")
            .replace("[", "\\[")
            .replace("`", "\\`")
            .replace("=", "\\=")
            .replace("-", "\\-")
    }

    pub fn new(b: Bot) -> TelegramNotificationSender {
        TelegramNotificationSender { bot: b }
    }
}

#[async_trait::async_trait]
impl NotificationSender for TelegramNotificationSender {
    async fn send_notification(&self, x: NotificationData) {
        let text = format!(
            "endpoint={};outage={};is_first={};admin={};http_address={}",
            x.endpoint, x.outage_id, x.is_first, x.admin, x.http_address
        );
        log::info!(
            "Attempting to concat {} by chat {}",
            x.admin,
            x.telegram_contact_id
        );
        let user_id = UserId(x.telegram_contact_id.parse().unwrap());
        match self
            .bot
            .send_message(user_id, Self::prepare_telegram_msg(text))
            .parse_mode(ParseMode::MarkdownV2)
            .send()
            .await
        {
            Ok(_) => log::info!("Message sent successfully"),
            Err(e) => log::error!("Failed to send message: {}", e),
        }
    }
}

fn parse_response(input: &str) -> Option<ResponseData> {
    let mut endpoint = None;
    let mut admin = None;
    let mut is_first = None;
    let mut outage_id = None;

    log::info!("Received {} telegram response", input);

    for part in input.split(';') {
        let key_value: Vec<&str> = part.trim().split('=').collect();
        if key_value.len() == 2 {
            match key_value[0].trim() {
                "endpoint" => endpoint = Some(key_value[1].trim().to_string()),
                "admin" => admin = Some(key_value[1].trim().to_string()),
                "is_first" => is_first = Some(key_value[1].trim().to_string()),
                "outage" => outage_id = Some(key_value[1].trim().to_string()),
                _ => (), // Unknown key
            }
        } else {
            () // Invalid key-value pair
        }
    }
    log::debug!(
        "Response after parse {:?} {:?} {:?} {:?}",
        endpoint,
        admin,
        is_first,
        outage_id
    );

    // Check if all required values are present
    if let (Some(endpoint), Some(admin), Some(is_first), Some(outage_id)) =
        (endpoint, admin, is_first, outage_id)
    {
        Some(ResponseData {
            admin: admin,
            outage_id: Uuid::parse_str(outage_id.as_str()).unwrap(),
            endpoint: endpoint.parse::<EndpointId>().unwrap(),
            is_first: is_first.parse().unwrap(),
        })
    } else {
        None // Missing one or more required keys
    }
}

impl TelegramNotificationReceiver {
    pub fn new(b: Bot, sender: Sender<ResponseData>) -> TelegramNotificationReceiver {
        TelegramNotificationReceiver {
            bot: b,
            sender: sender,
        }
    }

    async fn handle_reply(
        msg: Message,
        _bot: Bot,
        s_s: Sender<ResponseData>,
    ) -> ResponseResult<()> {
        if let Some(response) = msg
            .reply_to_message()
            .and_then(|msg| msg.text())
            .and_then(|x| parse_response(x))
        {
            s_s.send(response).await.unwrap();
        };
        Ok(())
    }
}

#[async_trait::async_trait]
impl ResponseListener for TelegramNotificationReceiver {
    async fn listen_for_responses(&self) {
        let handler = Update::filter_message().endpoint(
            |bot: Bot, sender: Sender<ResponseData>, msg: Message| async move {
                Self::handle_reply(msg, bot, sender).await
            },
        );

        Dispatcher::builder(self.bot.clone(), handler)
            .dependencies(dptree::deps![self.sender.clone()])
            .build()
            .dispatch()
            .await;
    }
}

pub fn create_telegram_bot() -> Bot {
    Bot::new("6886711339:AAGtn-uPuu2dHi4Y4KmJF47ovymj4XCAes4")
}

pub fn create_notification_sender_and_receiver(
    s: Sender<ResponseData>,
) -> (TelegramNotificationSender, TelegramNotificationReceiver) {
    let b = create_telegram_bot();
    (
        TelegramNotificationSender::new(b.clone()),
        TelegramNotificationReceiver::new(b, s),
    )
}
