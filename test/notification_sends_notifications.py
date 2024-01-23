import subprocess
import socket
import threading
import os
import time

tcp_server_port = 12345 

def check_result_code(result):
    if result.returncode != 0:
        print(f'Error running parallel script. Return code: {result.returncode}')
        exit(1)

def create_tables():
    script_path = '../python_postgres_setup/create_tables.py'
    arguments = ['python', script_path, '--drop-tables']
    check_result_code(subprocess.run(arguments))

def add_admins():
    script_path = '../configuration_scripts/add_admin.py'
    arguments_1 = ['python', script_path, '--admin-id', "Jacek", "--telegram-contact-id", "1219124635", "--phone", "2137", "--email", "matiutd888@gmail.com"]
    check_result_code(subprocess.run(arguments_1))
    arguments_2 = ['python', script_path, '--admin-id', "Mateusz", "--telegram-contact-id", "1219124635", "--phone", "2137", "--email", "matiutd888@gmail.com"]
    check_result_code(subprocess.run(arguments_2))

def add_endpoint():
    script_path = '../configuration_scripts/add_endpoint.py'
    arguments = ['python', script_path, '--http-address', "localhost:2137", "--primary-admin", "Mateusz", "--secondary-admin", "Jacek", "--response-duration", "10s", "--is-down",   "True", "--outage-id", "4d774191-ae6a-4acb-be73-369f8f1489cd"]
    check_result_code(subprocess.run(arguments))
    
def setup_db():
    create_tables()
    add_admins()
    add_endpoint()

def start_tcp_server(port):
    NOTIFICATION_SIZE = 2048
    N_NOTIFICATIONS = 2

    server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server_socket.bind(('localhost', port))
    server_socket.listen(1)
    print(f'TCP server listening on port {port}')

    connection, address = server_socket.accept()
    with connection:
        data = b''
        
        for _ in range(N_NOTIFICATIONS):  # Read for 2 minutes
            chunk = connection.recv(NOTIFICATION_SIZE)
            if not chunk:
                break
            data += chunk

        data_received = data.decode()
        print(f'Received data from client: {data_received}')
        
        # Assertion to check if the decoded data contains the string "jacek"
        assert 'jacek' in data_received.lower(), 'Assertion failed: "jacek" not found in the data.'
        assert 'mateusz' in data_received.lower(), 'Assertion failed: "mateusz" not found in the data.'
        
def run_cargo_app(cargo_dir):
    # Set the working directory to the specified directory for the Cargo app
    os.chdir(cargo_dir)
    return subprocess.Popen(['cargo', 'r', '--bin', 'notification', '--', '--notify-tcp', f"localhost:{tcp_server_port}"])

if __name__ == "__main__":
    # Task 1: Run parallel Python script
    parallel_script_thread = threading.Thread(target=setup_db)
    parallel_script_thread.start()
    parallel_script_thread.join()  # Wait for the parallel script to finish

    tcp_server_thread = threading.Thread(target=start_tcp_server, args=(tcp_server_port,))
    tcp_server_thread.start()

    # Task 3: Run Cargo app from a different directory
    cargo_app_directory = '../services/notification/'  # Replace with the actual path
    cargo_app = run_cargo_app(cargo_app_directory)
    

    # Wait for the TCP server to finish
    tcp_server_thread.join()

    # Task 4: Print output from the TCP server
    print('Finished successfully')
    # You may add code here to print or process the data received from the TCP server.

    cargo_app.terminate()
    