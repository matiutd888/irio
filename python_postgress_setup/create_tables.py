import psycopg2
from schemas import DATABASE_SETUP_QUERIES

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
    for query in DATABASE_SETUP_QUERIES:
        db_cursor.execute(query)
    db_connection.commit()
    print("Database setup completed successfully!")
except Exception as e:
    db_connection.rollback()
    print(f"Error during database setup: {e}")
finally:
    db_cursor.close()
    db_connection.close()
