use crate::{
    *
};

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
    async fn get_endpoints_to_process() -> Vec<EndpointData>;
}

// Send notification to given
#[async_trait::async_trait]
pub trait NotificationSender: Send + Sync {
    async fn send_notification(x: NotificationData);
}



#[async_trait::async_trait]
pub trait ResponseConsumer: Send + Sync {
    // If still is down - 
    async fn consume_response(response: ResponseData);
}

pub struct NotificationData {
    admin: AdminId,
    outage_id: OutageId,
    endpoint: EndpointId,
    contact_id: ContactId
}

pub struct ResponseData {
    admin: AdminId,
    outage_id: OutageId,
    endpoint: EndpointId
}
