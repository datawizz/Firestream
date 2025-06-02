
import numpy as np
from typing import Type
from pydantic import BaseModel, Field
from petastorm.unischema import Unischema, UnischemaField











def pydantic_to_unischema(model: Type[BaseModel], schema_name: str) -> Unischema:
    unischema_fields = []

    def process_field(name, field):
        if issubclass(field.type_, BaseModel):
            nested_unischema = pydantic_to_unischema(field.type_, f"{name}_Unischema")
            for nested_field in nested_unischema.fields:
                unischema_fields.append(nested_field)
        else:
            numpy_dtype = field.type_ if field.type_ is not np.ndarray else field.extra.get("numpy_dtype")
            shape = field.extra.get("shape", ())
            codec = field.extra.get("codec")
            nullable = field.allow_none
            unischema_field = UnischemaField(name, numpy_dtype, shape, codec, nullable)
            unischema_fields.append(unischema_field)

    for name, field in model.__fields__.items():
        process_field(name, field)

    return Unischema(schema_name, unischema_fields)





def test_pydantic_to_unischema_nested():
    class NestedModel(BaseModel):
        nested_field: float = Field(..., title="Nested Field", numpy_dtype=np.float32)

    class ExampleModel(BaseModel):
        name: str = Field(..., title="Name", numpy_dtype='<U64')
        age: int = Field(..., title="Age", ge=0, numpy_dtype=np.int32)
        nested_model: NestedModel = Field(..., title="Nested Model")

    # Convert the Pydantic model to a Unischema
    example_unischema = pydantic_to_unischema(ExampleModel, "ExampleUnischema")

    # Check if the Unischema has the expected fields
    expected_fields = [
        UnischemaField("name", np.dtype('<U64'), (), None, False),
        UnischemaField("age", np.int32, (), None, False),
        UnischemaField("nested_field", np.float32, (), None, False),
    ]

    assert example_unischema.fields == expected_fields, "Unischema fields do not match the expected fields"



if __name__ == "__main__":


    # Run the test
    test_pydantic_to_unischema_nested()





