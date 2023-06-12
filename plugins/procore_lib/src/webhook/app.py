from pydantic import BaseModel, Field
from datetime import datetime
from typing import Optional

class Metadata(BaseModel):
    source_application_id: Optional[int]
    source_company_id: int
    source_operation_id: Optional[int]
    source_project_id: int
    source_user_id: int

class EventType(str):
    CREATE = "create"
    UPDATE = "update"
    DELETE = "delete"

class Event(BaseModel):
    api_version: str = Field(..., alias="api_version")
    company_id: int = Field(..., alias="company_id")
    event_type: EventType = Field(..., alias="event_type")
    id: int = Field(..., alias="id")
    metadata: Metadata = Field(..., alias="metadata")
    project_id: int = Field(..., alias="project_id")
    resource_id: int = Field(..., alias="resource_id")
    resource_name: str = Field(..., alias="resource_name")
    timestamp: datetime = Field(..., alias="timestamp")
    ulid: str = Field(..., alias="ulid")
    user_id: int = Field(..., alias="user_id")

# Example data for validation
example_data = {
    "api_version": "v2",
    "company_id": 6452,
    "event_type": "create",
    "id": 11301387632,
    "metadata": {
        "source_application_id": None,
        "source_company_id": 6452,
        "source_operation_id": None,
        "source_project_id": 219542,
        "source_user_id": 1469242
    },
    "project_id": 219542,
    "resource_id": 245510722,
    "resource_name": "Purchase Order Contract Line Items",
    "timestamp": "2020-07-20T15:22:01.353581Z",
    "ulid": "01EDPD2J6SBY1163N77H95GW09",
    "user_id": 1469242
}

event = Event(**example_data)
print(event)


