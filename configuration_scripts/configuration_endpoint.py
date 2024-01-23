from fastapi import FastAPI
from add_admin import (
    add_admin_data,
    delete_admin_data,
    establish_db_connection,
    update_admin_data,
)
from configuration_types import AddRequest, UpdateRequest, DeleteRequest

app = FastAPI()
db_connection, db_cursor = establish_db_connection()


def get_dict_from_body(body: AddRequest | DeleteRequest | UpdateRequest):
    return {key: value for key, value in body.dict().items() if value is not None}


@app.post("/add/")
def add_admin(body: AddRequest):
    admin_data = get_dict_from_body(body)
    try:
        add_admin_data(db_cursor, admin_data)
        db_connection.commit()
    except Exception as e:
        db_connection.rollback()
        print(f"Error during admin data operation: {e}")


@app.post("/update/")
def update_admin(body: UpdateRequest):
    admin_data = get_dict_from_body(body)
    try:
        update_admin_data(db_cursor, admin_data)
        db_connection.commit()
    except Exception as e:
        db_connection.rollback()
        print(f"Error during admin data operation: {e}")


@app.post("/delete/")
def delete_admin(body: DeleteRequest):
    admin_data = get_dict_from_body(body)
    try:
        delete_admin_data(db_cursor, admin_data)
        db_connection.commit()
    except Exception as e:
        db_connection.rollback()
        print(f"Error during admin data operation: {e}")


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(app, host="127.0.0.1", port=8000)
