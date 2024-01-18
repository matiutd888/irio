use std::sync::Arc;

use anyhow::{Result, anyhow};
use teloxide::{
    payloads::SendMessageSetters,
    requests::{Request, Requester, ResponseResult},
    types::{ParseMode, UserId, Message},
    Bot,
};

use crate::lib::{NotificationData, NotificationSender, ResponseConsumer, ResponseData};

pub struct TelegramNotificationSender {
    bot: Arc<Bot>,
}

pub struct TelegramReceiver {
    bot: Arc<Bot>,
    consumer: Arc<dyn ResponseConsumer>,
}

impl TelegramNotificationSender {
    pub fn new(b: Arc<Bot>) -> TelegramNotificationSender {
        TelegramNotificationSender { bot: b }
    }
}

#[async_trait::async_trait]
impl NotificationSender for TelegramNotificationSender {
    async fn send_notification(&self, x: NotificationData) {
        let text = format!(
            "endpoint={};outage={};is_first={}",
            x.endpoint, x.outage_id, x.is_first
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

// fn parse_string(input: &str) -> Option<(String, String, String)> {
//     let mut endpoint = None;
//     let mut admin = None;
//     let mut owner = None;

//     for part in input.split(';') {
//         let key_value: Vec<&str> = part.trim().split('=').collect();
//         if key_value.len() == 2 {
//             match key_value[0].trim() {
//                 "endpoint" => endpoint = Some(key_value[1].trim().to_string()),
//                 "admin" => admin = Some(key_value[1].trim().to_string()),
//                 "owner" => owner = Some(key_value[1].trim().to_string()),
//                 _ => return None, // Unknown key
//             }
//         } else {
//             return None; // Invalid key-value pair
//         }
//     }

//     // Check if all required values are present
//     if let (Some(endpoint), Some(admin), Some(owner)) = (endpoint, admin, owner) {
//         Some((endpoint, admin, owner))
//     } else {
//         None // Missing one or more required keys
//     }
// }


impl TelegramReceiver {
    pub fn new(b: Arc<Bot>, consumer: Arc<dyn ResponseConsumer>) -> TelegramReceiver {
        TelegramReceiver {
            bot: b,
            consumer: consumer,
        }
    }

    async fn listen_for_replies(&self) {
        teloxide::repl(self.bot.clone(), |bot: Bot, msg: Message| async move {
            let msg = msg.reply_to_message()
            .and_then(|msg| {
              msg.text()
            }).and_then(|x| {
                x.pa
            });
            
            ResponseResult::<()>::Ok(())
        })
        .await;
    }
} 



pub fn create_telegram_bot() -> Arc<Bot> {
    Arc::new(Bot::new("6886711339:AAGtn-uPuu2dHi4Y4KmJF47ovymj4XCAes4"))
}
