#[async_trait::async_trait]
pub trait DBQueryExecutor: Send + Sync {
    // Read all endpoints that
    // 1. Are down
    // 2. 
    async fn get_endpoints_to_process() -> Vec<EndpointData>;
}

#[async_trait::async_trait]
pub trait NotificationSender: Send + Sync {
    async fn send_notification(x: NotificationData);
}


#[async_trait::async_trait]
pub trait ResponseConsumer: Send + Sync {
    async fn consume_response(response: ResponseData);
}

pub struct NotificationData {

}

pub struct ResponseData {

}

pub struct EndpointData {

}