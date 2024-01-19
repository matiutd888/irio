use chrono::NaiveDateTime;
use sqlx::{postgres::types::PgInterval, FromRow};
use uuid::Uuid;

pub type AdminId = String;
pub type ContactId = String;
pub type EndpointId = i32;
pub type OutageId = Uuid;
pub type MyTime = NaiveDateTime;
pub type MyDuration = PgInterval;
pub type ServiceInstanceId = Uuid;

#[derive(Debug, FromRow)]
pub struct EndpointData {
    pub endpoint_id: EndpointId,
    pub http_address: String,
    pub is_down: bool,
    pub outage_id: Option<OutageId>,
    pub ntf_is_being_handled: bool,
    pub ntf_is_being_handled_timestamp: Option<MyTime>,
    pub ntf_is_being_handled_service_id: Option<ServiceInstanceId>,
    pub ntf_is_first_notification_sent: bool,
    pub ntf_first_notification_sent_timestamp: Option<MyTime>,
    pub ntf_is_second_notification_sent: bool,
    pub conf_primary_admin: AdminId,
    pub conf_secondary_admin: AdminId,
    pub conf_allowed_response_duration: MyDuration,
    pub ntf_first_responded: bool,
}

#[derive(Debug, FromRow, Clone)]
pub struct Admin {
    pub admin_id: AdminId,
    pub telegram_contact_id: ContactId,
    pub phone_number: String,
    pub email_address: String,
}
