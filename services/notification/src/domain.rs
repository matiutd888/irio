use anyhow::Error;
use sqlx::{FromRow, Decode, postgres::types::PgInterval};
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate, NaiveDateTime};
use std::{time::Duration, sync::Arc};

pub type AdminId = Uuid;
pub type ContactId = String;
pub type EndpointId = String;
pub type OutageId = Uuid;
pub type MyTime = NaiveDateTime;
pub type MyDuration = PgInterval;
pub type ServiceInstanceId = Uuid;

#[derive(Debug, FromRow)]
pub struct EndpointData {
    pub endpoint_id: EndpointId,
    pub is_down: bool,
    pub outage_id: Option<OutageId>,
    pub ntf_is_being_handled: bool,
    pub ntf_is_being_handled_timestamp: Option<MyTime>, 
    pub ntf_is_being_handled_service_id: Option<ServiceInstanceId>,
    pub ntf_is_first_notification_sent: bool,
    pub ntf_first_notification_sent_timestamp: Option<MyTime>,
    pub ntf_is_second_notification_sent: bool,
    pub ntf_primary_admin: AdminId,
    pub ntf_secondary_admin: AdminId,
    pub ntf_allowed_response_duration: MyDuration,
    pub ntf_responded: bool,
}

pub struct Admins {
    pub admin_id: AdminId,
    pub contact_id: ContactId
}