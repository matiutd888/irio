from pydantic import BaseModel


class AddRequest(BaseModel):
    admin_id: str
    telegram_contact_id: str
    phone_number: str
    email_address: str


class UpdateRequest(BaseModel):
    admin_id: str
    telegram_contact_id: str = None
    phone_number: str = None
    email_address: str = None


class DeleteRequest(BaseModel):
    admin_id: str
