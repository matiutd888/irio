CREATE_ADMIN_TABLE_DB_QUERY = """
CREATE TABLE IF NOT EXISTS admin (
    admin_id VARCHAR(255) PRIMARY KEY,
    telegram_contact_id VARCHAR(255) NOT NULL,
    phone_number VARCHAR(20) NOT NULL,
    email_address VARCHAR(255) NOT NULL,
    is_removed BOOLEAN NOT NULL
);
"""

CREATE_ENDPOINT_DATA_DB_QUERY = """
CREATE TABLE IF NOT EXISTS endpoint_data (
    endpoint_id SERIAL PRIMARY KEY,
    http_address VARCHAR(255) NOT NULL,
    is_down BOOLEAN NOT NULL,
    outage_id UUID,
    ntf_is_being_handled BOOLEAN NOT NULL,
    ntf_is_being_handled_timestamp TIMESTAMP,
    ntf_is_being_handled_service_id UUID,
    ntf_is_first_notification_sent BOOLEAN NOT NULL,
    ntf_first_notification_sent_timestamp TIMESTAMP,
    ntf_is_second_notification_sent BOOLEAN NOT NULL,
    conf_primary_admin VARCHAR(255) REFERENCES admin(admin_id) NOT NULL,
    conf_secondary_admin VARCHAR(255) REFERENCES admin(admin_id) NOT NULL,
    conf_allowed_response_duration INTERVAL NOT NULL,
    ntf_first_responded BOOLEAN NOT NULL,
    is_removed BOOLEAN NOT NULL
);
"""

DATABASE_SETUP_QUERIES = [CREATE_ADMIN_TABLE_DB_QUERY, CREATE_ENDPOINT_DATA_DB_QUERY]
