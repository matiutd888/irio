import os
import argparse
import psycopg2
from datetime import datetime
import uuid

def add_endpoint(cursor, endpoint_data):
    try:
        cursor.execute("""
            INSERT INTO endpoint_data (
                http_address,
                is_down,
                outage_id,
                ntf_is_being_handled,
                ntf_is_being_handled_timestamp,
                ntf_is_being_handled_service_id,
                ntf_is_first_notification_sent,
                ntf_first_notification_sent_timestamp,
                ntf_is_second_notification_sent,
                conf_primary_admin,
                conf_secondary_admin,
                conf_allowed_response_duration,
                ntf_first_responded
            ) VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s)
        """, (
            endpoint_data['http_address'],
            endpoint_data['is_down'],
            endpoint_data['outage_id'],
            endpoint_data['ntf_is_being_handled'],
            endpoint_data['ntf_is_being_handled_timestamp'],
            endpoint_data['ntf_is_being_handled_service_id'],
            endpoint_data['ntf_is_first_notification_sent'],
            endpoint_data['ntf_first_notification_sent_timestamp'],
            endpoint_data['ntf_is_second_notification_sent'],
            endpoint_data['conf_primary_admin'],
            endpoint_data['conf_secondary_admin'],
            endpoint_data['conf_allowed_response_duration'],
            endpoint_data['ntf_first_responded']
        ))

        print("Endpoint added successfully!")

    except Exception as e:
        print(f"Error adding endpoint: {e}")

def main():
    parser = argparse.ArgumentParser(description="Add an endpoint to the endpoint_data table.")
    parser.add_argument('--http-address', type=str, required=True, help='HTTP address of the endpoint')
    parser.add_argument('--primary-admin', type=int, required=True, help='Primary admin ID')
    parser.add_argument('--secondary-admin', type=int, required=True, help='Secondary admin ID')
    parser.add_argument('--response-duration', type=str, required=True, help='Allowed response duration')

    args = parser.parse_args()

    endpoint_data_to_add = {
        'http_address': args.http_address,
        'is_down': False,
        'outage_id': None,
        'ntf_is_being_handled': False,
        'ntf_is_being_handled_timestamp': None,
        'ntf_is_being_handled_service_id': None,
        'ntf_is_first_notification_sent': False,
        'ntf_first_notification_sent_timestamp': None,
        'ntf_is_second_notification_sent': False,
        'conf_primary_admin': args.primary_admin,
        'conf_secondary_admin': args.secondary_admin,
        'conf_allowed_response_duration': args.response_duration,
        'ntf_first_responded': False
    }

    db_params = {
        'host': 'localhost',
        'port': 5432,
        'database': os.environ["POSTGRES_DB"],
        'user': os.environ["POSTGRES_USER"],
        'password': os.environ["POSTGRES_PASSWORD"]
    }

    with psycopg2.connect(**db_params) as connection:
        with connection.cursor() as cursor:
            add_endpoint(cursor, endpoint_data_to_add)

if __name__ == "__main__":
    main()
