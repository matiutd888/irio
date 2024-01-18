CREATE_ADMIN_TABLE_DB_QUERY = """
CREATE TABLE IF NOT EXISTS admin (
    admin_id UUID PRIMARY KEY,
    contact_id VARCHAR(255) NOT NULL
);
"""

CREATE_ENDPOINT_DATA_DB_QUERY = """
CREATE TABLE IF NOT EXISTS endpoint_data (
    endpoint_id VARCHAR(255) PRIMARY KEY,
    is_down BOOLEAN NOT NULL,
    outage_id UUID,
    ntf_is_being_handled BOOLEAN NOT NULL,
    ntf_is_being_handled_timestamp TIMESTAMP,
    ntf_is_being_handled_service_id UUID,
    ntf_is_first_notification_sent BOOLEAN NOT NULL,
    ntf_first_notification_sent_timestamp TIMESTAMP,
    ntf_is_second_notification_sent BOOLEAN NOT NULL,
    conf_primary_admin UUID REFERENCES admin(admin_id) NOT NULL,
    conf_secondary_admin UUID REFERENCES admin(admin_id) NOT NULL,
    conf_allowed_response_duration INTERVAL NOT NULL,
    ntf_first_responded BOOLEAN NOT NULL
);
"""

DATABASE_SETUP_QUERIES = [CREATE_ADMIN_TABLE_DB_QUERY, CREATE_ENDPOINT_DATA_DB_QUERY]
