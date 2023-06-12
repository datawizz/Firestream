from typing import List, Optional
from pydantic import BaseModel


class Attachment(BaseModel):
    id: int
    name: str
    url: str
    filename: str


class WorkflowPermanentLog(BaseModel):
    id: int
    activity: str
    attachments: List[Attachment]
    ball_in_court_duration: str
    comments: str
    created_at: str
    performed_by: str
    user_role: str


class ErrorResponse(BaseModel):
    code: int
    message: str
    fields: Optional[str] = None
    reason: Optional[str] = None
    error: Optional[str] = None


class Parameter(BaseModel):
    name: str
    in_: str
    description: str
    required: bool
    schema: dict


class Get(BaseModel):
    summary: str
    description: str
    parameters: List[Parameter]
    responses: dict
    tags: List[str]
    operationId: str


class APIModel(BaseModel):
    parameters: List[Parameter]
    get: Get



from typing import List, Optional
from pydantic import BaseModel


class Project(BaseModel):
    name: str
    id: int


class Assignee(BaseModel):
    id: int
    name: str


class WorkflowUserRole(BaseModel):
    id: int
    name: str
    assignee: Assignee


class WorkflowActivity(BaseModel):
    id: int
    name: str
    workflow_user_role: WorkflowUserRole
    perform_activity: dict


class WorkflowState(BaseModel):
    id: int
    name: str
    status: str


class Workflow(BaseModel):
    id: int
    name: Optional[str]
    description: Optional[str]
    class_name: str
    created_at: str
    updated_at: str
    domain: str


class WorkflowInstance(BaseModel):
    id: int
    becomes_overdue_at: Optional[str]
    current_state_set_at: str
    workflowed_object_id: int
    workflowed_object_type: str
    project: Optional[Project]
    current_workflow_activities: List[WorkflowActivity]
    current_workflow_state: WorkflowState
    workflow: Workflow


class ErrorResponse(BaseModel):
    code: int
    message: str
    fields: Optional[str] = None
    reason: Optional[str] = None
    error: Optional[str] = None


class Parameter(BaseModel):
    name: str
    in_: str
    description: str
    required: bool
    schema: dict


class Get(BaseModel):
    summary: str
    description: str
    responses: dict
    tags: List[str]
    operationId: str


class APIModel(BaseModel):
    parameters: List[Parameter]
    get: Get

