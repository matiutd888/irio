# import pytest


from time import sleep
from urllib import request
from telethon import TelegramClient, events, sync
import requests
import schemas
import psycopg2

TEST_SERVICE_URL = 'http://35.202.117.225:8001'
# def get_
import os
api_id = os.environ['API_ID']
api_hash = os.environ['API_HASH']
client = TelegramClient('session_name', api_id, api_hash)
client.start()

def execute_queries(cursor, queries):
    for query in queries:
        cursor.execute(query)

def drop_all_tables(cursor):
    cursor.execute("DROP SCHEMA public CASCADE; CREATE SCHEMA public;")

def get_db_params():
    return {
        "host": os.environ["POSTGRES_HOSTNAME"],
        "port": int(os.environ["POSTGRES_PORT"]),
        "database": os.environ["POSTGRES_DB"],
        "user": os.environ["POSTGRES_USER"],
        "password": os.environ["POSTGRES_PASSWORD"],
    }
def establish_db_connection():
    db_params = get_db_params()

    db_connection = psycopg2.connect(**db_params)
    db_cursor = db_connection.cursor()

    return db_connection, db_cursor

conn, cursor = establish_db_connection()

def acknowledge_alert(client: TelegramClient):
    messages = client.get_messages('alertingPlatformTestBot', limit=1)
    messages[0].reply('ack')

def check_alert(client: TelegramClient, enpdoint_id, admin, http_address, is_first) -> bool:
    messages = client.get_messages('alertingPlatformTestBot', limit=2)
    print(messages[0].message)
    tokens = messages[0].message.split(';')
    return tokens[0] == 'endpoint=' + enpdoint_id and tokens[2] == "is_first=" + str(is_first).lower() and tokens[3] == "admin=" + admin and tokens[4] == "http_address=" + http_address

def turn_off_service():
    requests.post(f'{TEST_SERVICE_URL}/shutdown')

def turn_on_service():
    requests.post(f'{TEST_SERVICE_URL}/turnon')

def service_is_up() -> bool:
    return requests.get(f'{TEST_SERVICE_URL}/status').status_code == 200 

def get_endpoint_data_from_db(endpoint_id):
    data = cursor.execute(f"SELECT * FROM endpoints WHERE endpoint_id = '{endpoint_id}'")
    conn.commit()
    if len(data) == 0:
        return None
    return data[0]

def add_admin_data(cursor, admin):
    try:
        if all(
            key in admin.keys()
            for key in [
                "admin_id",
                "telegram_contact_id",
                "phone_number",
                "email_address",
            ]
        ):
            cursor.execute(
                "INSERT INTO admin (admin_id, telegram_contact_id, phone_number, email_address, is_removed) VALUES (%s, %s, %s, %s, %s) RETURNING admin_id",
                (
                    admin["admin_id"],
                    admin["telegram_contact_id"],
                    admin["phone_number"],
                    admin["email_address"],
                    False
                ),
            )
            admin_id = cursor.fetchone()[0]
            print(f"Admin added successfully with admin_id: {admin_id}")
        else:
            print(
                "Error: Incomplete admin information. Provide admin_id, telegram_contact_id, phone_number, and email_address for each admin."
            )
    except Exception as e:
        print(f"Error adding admin: {e}")

def add_endpoint_data(cursor, endpoint_data):
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
                ntf_first_responded,
                is_removed,
                frequency,
                last_ping_time                       
            ) VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s)
            RETURNING endpoint_id
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
            endpoint_data['ntf_first_responded'],
            endpoint_data['is_removed'],
            endpoint_data['frequency'],
            endpoint_data['last_ping_time']
        ))

        endpoint_id = cursor.fetchone()[0]

        print(f"Endpoint {endpoint_id} added successfully!")

    except Exception as e:
        print(f"Error adding endpoint: {e}")
def add_endpoint_to_db(http_address, primary_admin, secondary_admin, response_duration, frequency):
    test_endpoint =  {
        'http_address': http_address,
        'is_down': False,
        'outage_id': None,
        'ntf_is_being_handled': False,
        'ntf_is_being_handled_timestamp': None,
        'ntf_is_being_handled_service_id': None,
        'ntf_is_first_notification_sent': False,
        'ntf_first_notification_sent_timestamp': None,
        'ntf_is_second_notification_sent': False,
        'conf_primary_admin': primary_admin,
        'conf_secondary_admin': secondary_admin,
        'conf_allowed_response_duration': f"{response_duration} SECONDS",
        'ntf_first_responded': False,
        'is_removed': False,
        'last_ping_time': None,
        'frequency': f"{frequency} SECONDS"
    }
    add_endpoint_data(cursor, test_endpoint)
    conn.commit()

def setup_db():
    drop_all_tables(cursor)
    execute_queries(cursor, schemas.DATABASE_SETUP_QUERIES)
    conn.commit()

def add_admin_to_db(admin_id, telegram_contact_id, phone_number, email_address):
    admin_data = {
        "admin_id": admin_id,
        "telegram_contact_id": telegram_contact_id,
        "phone_number": phone_number,
        "email_address": email_address,
    }
    add_admin_data(cursor, admin_data)

 
def test_send_2_messages_at_shutdown():
    setup_db()
    admin1 = 'TestAdmin'
    admin2 = 'TestAdmin2'
    turn_on_service()
    add_admin_to_db(admin1, '480068731', '1', '1')
    add_admin_to_db(admin2, '480068731', '1', '1')
    add_endpoint_to_db(f"{TEST_SERVICE_URL}/status", admin1, admin2, 10, 1)
    sleep(3)
    turn_off_service()
    check_count = 0
    while check_count < 10:
        sleep(1)
        print('Checking alert')
        is_alert_sent = check_alert(client, '1', admin1, f"{TEST_SERVICE_URL}/status", True)
        if is_alert_sent:
            print('Alert sent')
            break
        check_count += 1
    if check_count == 10:
        assert False
    sleep(12)
    assert check_alert(client, '1', admin2, f'{TEST_SERVICE_URL}/status', False)
    turn_on_service()
    
def test_send_1_messages_with_ack():
    setup_db()
    admin1 = 'TestAdmin'
    admin2 = 'TestAdmin2'
    turn_on_service()
    add_admin_to_db(admin1, '480068731', '1', '1')
    add_admin_to_db(admin2, '480068731', '1', '1')
    add_endpoint_to_db(f"{TEST_SERVICE_URL}/status", admin1, admin2, 10, 1)
    sleep(3)
    turn_off_service()
    check_count = 0
    while check_count < 10:
        sleep(1)
        print('Checking alert')
        is_alert_sent = check_alert(client, '1', admin1, f"{TEST_SERVICE_URL}/status", True)
        if is_alert_sent:
            print('Alert sent')
            acknowledge_alert(client)
            break
        check_count += 1
    if check_count == 10:
        assert False
    sleep(12)
    assert check_alert(client, '1', admin2, f'{TEST_SERVICE_URL}/status', False) == False
    turn_on_service()

