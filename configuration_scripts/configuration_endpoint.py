from fastapi import FastAPI
from add_admin import (
    add_admin_data,
    delete_admin_data,
    establish_db_connection,
    update_admin_data,
)
from add_endpoint import get_endpoint_from_dict, delete_endpoint_data, add_endpoint_data
from configuration_types import AddAdminRequest, AddEndpointRequest, DeleteEndpointRequest, UpdateAdminRequest, DeleteAdminRequest

app = FastAPI()
db_connection, db_cursor = establish_db_connection()


def get_dict_from_body(body: AddAdminRequest | DeleteAdminRequest | UpdateAdminRequest):
    return {key: value for key, value in body.dict().items() if value is not None}


@app.post("/add_admin/")
def add_admin(body: AddAdminRequest):
    admin_data = get_dict_from_body(body)
    try:
        add_admin_data(db_cursor, admin_data)
        db_connection.commit()
    except Exception as e:
        db_connection.rollback()
        print(f"Error during admin data operation: {e}")


@app.post("/update_admin/")
def update_admin(body: UpdateAdminRequest):
    admin_data = get_dict_from_body(body)
    try:
        update_admin_data(db_cursor, admin_data)
        db_connection.commit()
    except Exception as e:
        db_connection.rollback()
        print(f"Error during admin data operation: {e}")


@app.post("/delete_admin/")
def delete_admin(body: DeleteAdminRequest):
    admin_data = get_dict_from_body(body)
    try:
        delete_admin_data(db_cursor, admin_data)
        db_connection.commit()
    except Exception as e:
        db_connection.rollback()
        print(f"Error during admin data operation: {e}")

@app.post("/add_endpoint/")
def add_endpoint(body: AddEndpointRequest):
    req_data = get_endpoint_from_dict(body)
    try:
        add_endpoint_data(db_cursor, req_data)
        db_connection.commit()
    except Exception as e:
        db_connection.rollback()
        print(f"Error: during endpoint data operation: {e}")
    

@app.post("/delete_endpoint/")
def delete_endpoint(body: DeleteEndpointRequest):
    try:
        delete_endpoint_data(db_cursor, body.endpoint_id)
        db_connection.commit()
    except Exception as e:
        db_connection.rollback()
        print(f"Error: during endpoint data operation: {e}")
    

if __name__ == "__main__":
    import uvicorn

    uvicorn.run(app, host="0.0.0.0", port=8000)
