from datetime import datetime
from typing import List, Optional

from pydantic import BaseModel, Field


class Permission(BaseModel):
    can_update: bool = Field(..., description="Can Update")
    can_delete: bool = Field(..., description="Can Delete")


class Attachment(BaseModel):
    id: int
    filename: Optional[str] = Field(None, description=":filename to be deprecated, use :name")
    content_type: Optional[str] = Field(None, description="Content type of attachment")
    name: Optional[str] = Field(None, description="Name of attachment")
    url: str
    share_url: str = Field(..., example="https://example.com/prostore_local/...")
    viewable_type: str = Field(..., example="image")
    viewable_url: str = Field(..., example="https://example.com/...viewable_document_image_show?...")


class CreatedBy(BaseModel):
    id: int = Field(..., description="ID")
    login: str = Field(..., description="Email")
    name: str = Field(..., description="Name")


class AccidentLog(BaseModel):
    id: int = Field(..., description="ID")
    comments: Optional[str] = Field(None, description="Additional information about the accident")
    created_at: datetime = Field(..., description="Created at")
    date: str = Field(..., description="Date that the accident occurred")
    datetime: datetime = Field(..., description="Estimated UTC datetime of record")
    deleted_at: Optional[datetime] = Field(None, description="Deleted at")
    involved_name: str = Field(..., description="Name of the person involved in the accident")
    permissions: Optional[Permission] = Field(None, description="TBD")
    involved_company: str = Field(..., description="Name of the Company involved in the accident")
    position: int = Field(..., description="Order in which this entry was recorded for the day")
    time_hour: int = Field(..., description="Time of accident - hour", ge=0, le=23)
    time_minute: int = Field(..., description="Time of accident - minute", ge=0, le=59)
    updated_at: datetime = Field(..., description="Updated at")
    created_by: CreatedBy = Field(..., description="User who created the record")
    attachments: Optional[List[Attachment]] = Field(None, description="List of attachments to the record")
    
    class Config:
        schema_extra = {
            "example": {
                "id": 333675,
                "comments": "There was an accident on the roof",
                "created_at": "2012-10-23T21:39:40Z",
                "date": "2016-05-19",
                "datetime": "2016-05-19T12:00:00Z",
                "deleted_at": "2017-07-29T21:39:40Z",
                "involved_name": "4",
                "permissions": {"can_update": True, "can_delete": False},
                "involved_company": "Procore Technologies",
                "position": 142143,
                "time_hour": 10,
                "time_minute": 15,
                "updated_at": "2012-10-24T21:39:40Z",
                "created_by": {"id": 160586, "login": "carl.contractor@example.com", "name": "Carl Contractor"},
                "attachments": [
                    {
                        "id": 123,
                        "name": "attachment1.jpg",
                        "content_type": "image/jpeg",
                        "url": "https://example.com/image1.jpg",
                        "share_url": "https://example.com/prostore_local/...",
                        "viewable_type": "image",
                        "viewable_url": "https://example.com/...viewable_document_image_show?..."
                    },
                    {
                        "id": 456,
                        "name": "attachment2.png",
                        "content_type": "image/png",
                        "url": "https://example.com/image2.png",
                        "share_url": "https://example.com/prostore_local/...",
                        "viewable_type": "image",
                        "viewable_url": "https://example.com/...viewable_document_image_show?..."
                    }
                ]
            }
        }