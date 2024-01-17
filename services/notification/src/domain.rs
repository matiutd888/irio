use scylla::frame::value::Time;
use uuid::Uuid;

pub type AdminId = u64;
pub type Timestamp = u64;
pub type ContactId = str;
pub type EndpointId = Box<str>;
pub type OutageId = Uuid;

pub struct EndpointData {
    pub is_down: bool,
    pub outage_id: OutageId,
    pub ntf_is_being_handled: bool,
    pub ntf_is_being_handled_timestamp: bool,
    pub ntf_is_first_notification_sent: bool,
    pub ntf_first_notification_send_time: Timestamp,
    pub ntf_is_second_notification_sent: bool,
    pub ntf_primary_admin: AdminId,
    pub ntf_secondary_admin: AdminId,
    pub ntf_allowed_response_time: Time,
    pub ntf_first_responded: bool,
}

pub struct Admins {
    pub admin_id: AdminId,
    pub contact_id: ContactId
}