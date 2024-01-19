import psycopg2

def get_all_tables(cursor):
    cursor.execute("""
        SELECT table_name
        FROM information_schema.tables
        WHERE table_schema = 'public' AND table_type = 'BASE TABLE'
    """)
    return [table[0] for table in cursor.fetchall()]

def print_table_data(cursor, table_name):
    cursor.execute(f"SELECT * FROM {table_name}")
    columns = [desc[0] for desc in cursor.description]
    data = cursor.fetchall()

    print(f"\nTable: {table_name}")
    print(columns)
    for row in data:
        print(row)

def main():
    db_params = {
        'host': 'localhost',
        'port': 5432,
        'database': 'alerting_platform_db',
        'user': 'zolwiczek',
        'password': 'kaczusia'
    }

    db_connection = psycopg2.connect(**db_params)
    db_cursor = db_connection.cursor()

    try:
        tables = get_all_tables(db_cursor)
        if not tables:
            print("No tables found in the database.")
        else:
            for table in tables:
                print_table_data(db_cursor, table)

    except Exception as e:
        print(f"Error retrieving table data: {e}")

    finally:
        db_cursor.close()
        db_connection.close()

if __name__ == "__main__":
    main()