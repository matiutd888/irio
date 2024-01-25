from pydantic import BaseModel


class AddAdminRequest(BaseModel):
    admin_id: str
    telegram_contact_id: str
    phone_number: str
    email_address: str


class UpdateAdminRequest(BaseModel):
    admin_id: str
    telegram_contact_id: str = None
    phone_number: str = None
    email_address: str = None


class DeleteAdminRequest(BaseModel):
    admin_id: str

class AddEndpointRequest(BaseModel):
    http_address: str
    is_down: bool = False
    outage_id: str = None
    primary_admin: str
    secondary_admin: str
    response_duration: str

class DeleteEndpointRequest(BaseModel):
    endpoint_id: str
