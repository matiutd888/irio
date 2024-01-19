import os
import argparse
import psycopg2

def add_admin_data(cursor, admin_data):
    try:
        for admin in admin_data:
            cursor.execute(
                "INSERT INTO admin (telegram_contact_id, phone_number, email_address) VALUES (%s, %s, %s) RETURNING admin_id",
                (admin['telegram_contact_id'], admin['phone_number'], admin['email_address'])
            )
            admin_id = cursor.fetchone()[0]

            print(f"Admin added successfully with admin_id: {admin_id}")

    except Exception as e:
        print(f"Error adding admin: {e}")

def main():
    parser = argparse.ArgumentParser(description="Add a single admin to the admin table.")
    parser.add_argument('--contact-id', required=True, help='Contact ID of the admin')
    parser.add_argument('--phone', required=True, help='Phone number of the admin')
    parser.add_argument('--email', required=True, help='Email address of the admin')
    args = parser.parse_args()

    admin_data_to_add = [
        {'telegram_contact_id': args.telegram_contact_id, 'phone_number': args.phone, 'email_address': args.email}
    ]

    db_params = {
        'host': 'localhost',
        'port': int(os.environ["POSTGRES_PORT"]),
        'database': os.environ["POSTGRES_DB"],
        'user': os.environ["POSTGRES_USER"],
        'password': os.environ["POSTGRES_PASSWORD"]
    }

    db_connection = psycopg2.connect(**db_params)
    db_cursor = db_connection.cursor()

    try:
        add_admin_data(db_cursor, admin_data_to_add)
        db_connection.commit()

    except Exception as e:
        db_connection.rollback()
        print(f"Error during admin data insertion: {e}")

    finally:
        db_cursor.close()
        db_connection.close()

if __name__ == "__main__":
    main()
