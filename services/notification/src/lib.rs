use crate::*;

use self::domain::{AdminId, ContactId, EndpointData, EndpointId, OutageId};

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
}

// Send notification to given
#[async_trait::async_trait]
pub trait NotificationSender: Send + Sync {
    async fn send_notification(&self, x: NotificationData);
}

#[async_trait::async_trait]
pub trait ResponseConsumer: Send + Sync {
    // If still is down -
    async fn consume_response(&self, response: ResponseData);
}

// pub trait ResponseListener: Send + Sync {
//     async fn listen_for_responses() {

//     }
// }

pub struct NotificationData {
    pub admin: AdminId,
    pub outage_id: OutageId,
    pub endpoint: EndpointId,
    pub contact_id: ContactId,
    pub is_first: bool
}

pub struct ResponseData {
    pub admin: AdminId,
    pub outage_id: OutageId,
    pub endpoint: EndpointId,
    pub is_first: bool
}
