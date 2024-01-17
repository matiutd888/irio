use scylla::{Session, FromRow, cql_to_rust::FromCqlVal};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::{time::Duration, sync::Arc};

pub type AdminId = Uuid;
pub type ContactId = String;
pub type EndpointId = String;
pub type OutageId = Uuid;
pub type MyTime = DateTime<Utc>;
pub type MyDuration = i32;
pub type ServiceInstanceId = Uuid;

#[derive(Debug)]
pub struct EndpointData {
    pub is_down: bool,
    pub outage_id: OutageId,
    pub ntf_is_being_handled: bool,
    pub ntf_is_being_handled_timestamp: Option<MyTime>, 
    pub ntf_is_being_handled_service_id: Option<ServiceInstanceId>,
    pub ntf_is_first_notification_sent: bool,
    pub ntf_first_notification_sent_plus_allowed_response_time: Option<MyTime>,
    pub ntf_is_second_notification_sent: bool,
    pub ntf_primary_admin: AdminId,
    pub ntf_secondary_admin: AdminId,
    pub ntf_allowed_response_duration: MyDuration,
    pub ntf_first_responded: bool,
}

#[derive(Debug, FromRow)]
pub struct SomeInternalType {
    uuid_field: Uuid
}

// #[derive(Debug, FromRow)]
// pub struct EndpointDataDB {
//     pub is_down: bool,
//     pub outage_id: Uuid,
//     pub ntf_is_being_handled: bool,
//     pub ntf_is_being_handled_timestamp: Option<scylla::frame::value::Timestamp>, 
//     pub ntf_is_being_handled_service_id: Option<ServiceInstanceId>,
//     pub ntf_is_first_notification_sent: bool,
//     pub ntf_first_notification_sent_plus_allowed_response_time: Option<MyTime>,
//     pub ntf_is_second_notification_sent: bool,
//     pub ntf_primary_admin: AdminId,
//     pub ntf_secondary_admin: AdminId,
//     pub ntf_allowed_response_duration: MyDuration,
//     pub ntf_first_responded: bool,
// }


pub struct Admins {
    pub admin_id: AdminId,
    pub contact_id: ContactId
}