use std::sync::Arc;

use teloxide::{
    payloads::SendMessageSetters,
    requests::{Request, Requester},
    types::{ParseMode, UserId},
    Bot,
};

use crate::lib::{NotificationData, NotificationSender};

pub struct TelegramNotificationSender {
    bot: Arc<Bot>,
}

impl TelegramNotificationSender {
    pub fn new(b: Arc<Bot>) -> TelegramNotificationSender {
        TelegramNotificationSender { bot: b }
    }
}

#[async_trait::async_trait]
impl NotificationSender for TelegramNotificationSender {
    async fn send_notification(&self, x: NotificationData) {
        let text = format!("endpoint={};outage={}", x.endpoint, x.outage_id);

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

pub fn create_telegram_bot() -> Arc<Bot> {
    Arc::new(Bot::new("6886711339:AAGtn-uPuu2dHi4Y4KmJF47ovymj4XCAes4"))
}
