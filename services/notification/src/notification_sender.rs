use std::sync::Arc;

use teloxide::{
    dispatching::Dispatcher,
    payloads::SendMessageSetters,
    requests::{Request, Requester, ResponseResult},
    types::{Message, ParseMode, Update, UserId},
    Bot,
};

use teloxide::prelude::*;
use tokio::sync::mpsc::{channel, Sender};
use uuid::{uuid, Uuid};

use crate::lib::{NotificationData, NotificationSender, ResponseData};

#[derive(Debug, Clone)]
pub struct TelegramNotificationSender {
    bot: Bot,
}


#[derive(Debug, Clone)]
pub struct TelegramReceiver {
    bot: Bot,
    sender: Sender<ResponseData>,
}

impl TelegramNotificationSender {
    pub fn new(b: Bot) -> TelegramNotificationSender {
        TelegramNotificationSender { bot: b }
    }
}

#[async_trait::async_trait]
impl NotificationSender for TelegramNotificationSender {
    async fn send_notification(&self, x: NotificationData) {
        let text = format!(
            "endpoint={};outage={};is_first={};admin={}",
            x.endpoint, x.outage_id, x.is_first, x.admin
        );

        let user_id = UserId(x.contact_id.parse().unwrap());
        match self
            .bot
            .send_message(user_id, text)
            .parse_mode(ParseMode::MarkdownV2)
            .send()
            .await
        {
            Ok(_) => println!("Message sent successfully"),
            Err(e) => eprintln!("Failed to send message: {}", e),
        }
    }
}

fn parse_response(input: &str) -> Option<ResponseData> {
    let mut endpoint = None;
    let mut admin = None;
    let mut is_first = None;
    let mut outage_id = None;

    for part in input.split(';') {
        let key_value: Vec<&str> = part.trim().split('=').collect();
        if key_value.len() == 2 {
            match key_value[0].trim() {
                "endpoint" => endpoint = Some(key_value[1].trim().to_string()),
                "admin" => admin = Some(key_value[1].trim().to_string()),
                "is_first" => is_first = Some(key_value[1].trim().to_string()),
                "outage_id" => outage_id = Some(key_value[1].trim().to_string()),
                _ => return None, // Unknown key
            }
        } else {
            return None; // Invalid key-value pair
        }
    }

    // Check if all required values are present
    if let (Some(endpoint), Some(admin), Some(is_first), Some(outage_id)) =
        (endpoint, admin, is_first, outage_id)
    {
        Some(ResponseData {
            admin: Uuid::parse_str(admin.as_str()).unwrap(),
            outage_id: Uuid::parse_str(outage_id.as_str()).unwrap(),
            endpoint: endpoint,
            is_first: is_first.parse().unwrap(),
        })
    } else {
        None // Missing one or more required keys
    }
}

impl TelegramReceiver {
    pub fn new(b: Bot, sender: Sender<ResponseData>) -> TelegramReceiver {
        TelegramReceiver {
            bot: b,
            sender: sender,
        }
    }

    async fn handle_reply(
        msg: Message,
        bot: Bot,
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

    pub async fn listen_for_replies(&self) {
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

pub fn create_notification_sender_and_receiver(s: Sender<ResponseData>) -> (TelegramNotificationSender, TelegramReceiver) {
    let b = create_telegram_bot();
    (
        TelegramNotificationSender::new(b.clone()),
        TelegramReceiver::new(b, s)
    )
}