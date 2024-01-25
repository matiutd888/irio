import os
import argparse
import psycopg2


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


def update_admin_data(cursor, admin):
    try:
        admin_id = admin.get("admin_id")
        update_values = {
            key: admin[key]
            for key in admin.keys()
            if key != "admin_id" and admin[key] is not None
        }

        if admin_id and update_values:
            # Check if admin_id exists
            cursor.execute(
                "SELECT admin_id FROM admin WHERE admin_id = %s", (admin_id,)
            )
            existing_admin = cursor.fetchone()

            if existing_admin:
                update_query = "UPDATE admin SET "
                update_query += ", ".join(f"{key} = %s" for key in update_values.keys())
                update_query += " WHERE admin_id = %s AND is_removed = False"

                cursor.execute(
                    update_query, tuple(update_values.values()) + (admin_id,)
                )
                print(f"Admin with admin_id {admin_id} updated successfully")
            else:
                print(f"Error: Admin with admin_id {admin_id} does not exist.")
        else:
            print("Error: Provide admin_id and at least one field to update.")
    except Exception as e:
        print(f"Error updating admin: {e}")


def delete_admin_data(cursor, admin):
    try:
        admin_id = admin.get("admin_id")
        if admin_id:
            cursor.execute("UPDATE admin SET is_removed = True WHERE admin_id = %s", (admin_id,))
            print(f"Admin with admin_id {admin_id} deleted successfully")
        else:
            print("Error: Provide admin_id")
    except Exception as e:
        print(f"Error deleting admin: {e}")


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


def main():
    parser = argparse.ArgumentParser(description="Manage admins in the admin table.")
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--add", action="store_true", help="Add a new admin")
    group.add_argument("--update", action="store_true", help="Update an existing admin")
    group.add_argument("--delete", action="store_true", help="Delete an admin")
    parser.add_argument("--admin-id", required=True, help="Id of admin")
    parser.add_argument(
        "--telegram-contact-id", required=False, help="telegram contact ID of the admin"
    )
    parser.add_argument("--phone", required=False, help="Phone number of the admin")
    parser.add_argument("--email", required=False, help="Email address of the admin")
    args = parser.parse_args()

    admin_data = {
        "admin_id": args.admin_id,
        "telegram_contact_id": args.telegram_contact_id,
        "phone_number": args.phone,
        "email_address": args.email,
    }

    db_connection, db_cursor = establish_db_connection()

    try:
        if args.add:
            add_admin_data(db_cursor, admin_data)
        elif args.update:
            update_admin_data(db_cursor, admin_data)
        elif args.delete:
            delete_admin_data(db_cursor, admin_data)

        db_connection.commit()

    except Exception as e:
        db_connection.rollback()
        print(f"Error during admin data operation: {e}")

    finally:
        db_cursor.close()
        db_connection.close()


if __name__ == "__main__":
    main()
