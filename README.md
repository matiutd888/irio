# Project Name

## Setting Up the Database Docker Container

1. Navigate to the `local_docker` Directory:
   ```bash
   cd local_docker
   ```

2. Run the Docker Container:
   Execute the `run_postgres_container.sh` script to set up a PostgreSQL database container.
   ```bash
   ./run_postgres_container.sh
   ```

3. Configure Environmental Variables:
   Update the `.env` file in the `local_docker` directory with the necessary environmental variables, such as `DB_USER`, `DB_PASSWORD`, and `DB_NAME`.

## Python PostgreSQL Setup

1. Navigate to the `python_postgres_setup` Directory:
   ```bash
   cd python_postgres_setup
   ```

2. Run Python Script to Create Tables:
   Execute the `create_tables.py` script to create tables in the PostgreSQL database container.
   ```bash
   python create_tables.py
   ```

## Requirements
Make sure to install the required Python packages by running the following command in the root directory of the project:
```bash
pip install -r requirements.txt
```

### Environmental Variables in `.env`

- `DB_USER`: Database user
- `DB_PASSWORD`: Database password
- `DB_NAME`: Database name

### Note
Ensure that the PostgreSQL Docker container is running before executing the Python script.
