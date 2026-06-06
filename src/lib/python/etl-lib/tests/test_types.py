from dataclasses import dataclass
from datetime import date, datetime
from decimal import Decimal
from typing import Dict, List, Optional

import pyspark.sql.types as t
import pytest

from etl_lib._utils.types.types import (
    BoundDecimal,
    check_pyspark_struct,
    decimal,
    infer_spark_type,
    is_pyspark_class,
    long,
    struct,
    transform,
)


def test_infer_scalar_types():
    assert isinstance(infer_spark_type(str), t.StringType)
    assert isinstance(infer_spark_type(bool), t.BooleanType)
    assert isinstance(infer_spark_type(float), t.DoubleType)
    assert isinstance(infer_spark_type(int), t.IntegerType)
    assert isinstance(infer_spark_type(date), t.DateType)
    assert isinstance(infer_spark_type(datetime), t.TimestampType)
    assert isinstance(infer_spark_type(bytes), t.BinaryType)


def test_infer_decimal_with_default_precision():
    spark_type = infer_spark_type(Decimal)
    assert isinstance(spark_type, t.DecimalType)
    assert spark_type.precision == 36
    assert spark_type.scale == 6


def test_infer_bound_decimal():
    Money = decimal(prec=18, rounding=4)
    assert issubclass(Money, BoundDecimal)
    spark_type = infer_spark_type(Money)
    assert isinstance(spark_type, t.DecimalType)
    assert spark_type.precision == 18
    assert spark_type.scale == 4


def test_infer_long_typevar_maps_to_long_type():
    assert isinstance(infer_spark_type(long), t.LongType)


def test_infer_list_and_dict():
    array_type = infer_spark_type(List[int])
    assert isinstance(array_type, t.ArrayType)
    assert isinstance(array_type.elementType, t.IntegerType)

    map_type = infer_spark_type(Dict[str, float])
    assert isinstance(map_type, t.MapType)
    assert isinstance(map_type.keyType, t.StringType)
    assert isinstance(map_type.valueType, t.DoubleType)


def test_struct_decorator_and_transform():
    @struct
    @dataclass
    class Person:
        name: str
        age: int
        nickname: Optional[str]

    assert is_pyspark_class(Person)
    check_pyspark_struct(Person)

    schema = transform(Person)
    assert isinstance(schema, t.StructType)
    field_names = {f.name for f in schema.fields}
    assert field_names == {"name", "age", "nickname"}

    nickname_field = next(f for f in schema.fields if f.name == "nickname")
    assert nickname_field.nullable is True

    name_field = next(f for f in schema.fields if f.name == "name")
    assert name_field.nullable is False


def test_struct_requires_container():
    class NotAContainer:
        pass

    with pytest.raises(ValueError):
        struct(NotAContainer)


def test_infer_unknown_type_raises():
    class Unknown:
        pass

    with pytest.raises(TypeError):
        infer_spark_type(Unknown)
