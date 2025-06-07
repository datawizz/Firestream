from pydantic import BaseModel
from typing import List, Optional

class Child(BaseModel):
    name: str
    age: int

class Parent(BaseModel):
    name: str
    age: int
    children: List[Child]


from pydantic import BaseModel
from typing import Dict, Any

def get_schema_dict(model: BaseModel) -> Dict[str, Any]:
    schema = model.schema()

    for field, value in schema.get('properties', {}).items():
        if "$ref" in value:
            child_model_name = value["$ref"].split("/")[-1]
            child_model = model.__annotations__[field]
            value.update(get_schema_dict(child_model()))

    return schema

parent_schema = get_schema_dict(Parent)
print(parent_schema)


parent = Parent(name='John', age=40, children=[Child(name='Jack', age=10), Child(name='Jill', age=8)])
print(get_schema_dict(parent))
