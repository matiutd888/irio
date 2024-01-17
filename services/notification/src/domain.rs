pub type admin_id_t = u64;
pub type time_t = u64;
pub type contact_id_t = str;

pub struct Endpoint {
    pub is_down: bool,
    pub ntf_is_notification_sent: bool,
    pub ntf_is_being_handled: bool,
    pub ntf_is_being_handled_timestamp: bool,
    pub ntf_is_second_notification_sent: bool,
    pub ntf_primary_admin: admin_id_t,
    pub ntf_secondary_admin: admin_id_t,
    pub ntf_allowed_response_time: time_t,
    pub ntf_responded_first: bool,
    pub ntf_responded_second: bool
}

pub struct Admins {
    pub admin_id: admin_id_t,
    pub contact_id: contact_id_t
}