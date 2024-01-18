pub struct TelegramNotificationSender {

}

impl TelegramNotificationSender {
    pub fn new() -> TelegramNotificationSender {
        let bot = Bot::new("6886711339:AAGtn-uPuu2dHi4Y4KmJF47ovymj4XCAes4");

    }
}


// async fn send_message_with_id(bot: &Bot, user_id: UserId) {
//     let text = format!("new Message with {} sent", user_id);

//     match bot
//         .send_message(user_id, text)
//         .parse_mode(ParseMode::MarkdownV2)
//         .send()
//         .await
//     {
//         Ok(_) => println!("Message sent successfully"),
//         Err(e) => eprintln!("Failed to send message: {}", e),
//     }
// }
