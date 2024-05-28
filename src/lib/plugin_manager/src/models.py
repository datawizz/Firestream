from typing import List, Optional
from pydantic import BaseModel, validator
import os
import yaml

class ManifestModel(BaseModel):
    name: str
    path: str
    description: Optional[str] = None
    version: Optional[str] = None
    author: Optional[str] = None
    chart_dir: Optional[str] = None
    template_values: Optional[str] = None
    feature_flags: Optional[List[str]] = None
    dag_dir: Optional[str] = None

    @validator('chart_dir')
    def validate_chart_dir(cls, v):
        if v and not os.path.isdir(v):
            raise ValueError(f'{v} is not a valid directory')
        return v

    @validator('template_values')
    def validate_template_values(cls, v):
        if v and not os.path.isfile(v):
            raise ValueError(f'{v} is not a valid file')
        try:
            with open(v, 'r') as file:
                yaml.safe_load(file)
        except Exception:
            raise ValueError(f'{v} is not a valid YAML file')
        return v
