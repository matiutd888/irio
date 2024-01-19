import os
import argparse
import psycopg2
from schemas import DATABASE_SETUP_QUERIES

def execute_queries(cursor, queries):
    for query in queries:
        cursor.execute(query)

def drop_all_tables(cursor):
    cursor.execute("DROP SCHEMA public CASCADE; CREATE SCHEMA public;")

def main():
    parser = argparse.ArgumentParser(description="Setup the database and optionally drop existing tables.")
    parser.add_argument('--drop-tables', action='store_true', help='Drop all existing tables before setup')
    args = parser.parse_args()

    db_params = {
        'host': 'localhost',
        'port': 5432,
        'database': os.environ["POSTGRES_DB"],
        'user': os.environ["POSTGRES_USER"],
        'password': os.environ["POSTGRES_PASSWORD"]
    }

    db_connection = psycopg2.connect(**db_params)
    db_cursor = db_connection.cursor()

    try:
        if args.drop_tables:
            print("Dropping existing tables...")
            drop_all_tables(db_cursor)

        print("Setting up the database...")
        execute_queries(db_cursor, DATABASE_SETUP_QUERIES)

        db_connection.commit()
        print("Database setup completed successfully!")

    except Exception as e:
        db_connection.rollback()
        print(f"Error during database setup: {e}")

    finally:
        db_cursor.close()
        db_connection.close()

if __name__ == "__main__":
    main()
