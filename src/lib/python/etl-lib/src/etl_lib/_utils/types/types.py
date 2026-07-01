"""Type inference helpers bridging Python type hints to PySpark schemas.

Vanilla Python lacks fixed-width integer types and a single distinction
between ``date`` and ``datetime``. These helpers infer the appropriate Spark
DataType for an annotated class field so that a ``@struct``-decorated
NamedTuple or dataclass can be materialized as a ``StructType`` without the
caller writing schema definitions by hand.
"""

from datetime import date, datetime
from decimal import Decimal
from typing import Dict, Generic, List, Optional, Tuple, TypeVar, Union

import pyspark.sql.types as t

NoneType = type(None)
byte = TypeVar("byte")
short = TypeVar("short")
long = TypeVar("long")
T = TypeVar("T")
T_co = TypeVar("T_co", covariant=True)

__type_cache: Dict[str, type] = {}


class BoundDecimal:
    __constraints__: Tuple[int, int]


class FunctorLike(Generic[T_co]):
    __slots__ = ()


def decimal(prec: int, rounding: int) -> type:
    """Return (and memoize) a ``BoundDecimal`` subclass with the given precision."""
    name = f"BoundDecimal_{prec}_{rounding}"
    if name in __type_cache:
        return __type_cache[name]
    cls = type(name, (BoundDecimal, Decimal), {})
    cls.__constraints__ = (prec, rounding)
    __type_cache[name] = cls
    return cls


def is_pyspark_class(cls: type) -> bool:
    return getattr(cls, "__pyspark_struct__", None) is ...


def is_container(cls: type) -> bool:
    fields = getattr(cls, "_fields", None)
    dc_fields = getattr(cls, "__dataclass_fields__", None)
    annotations = getattr(cls, "__annotations__", None)
    return (isinstance(fields, tuple) or isinstance(dc_fields, dict)) and isinstance(
        annotations, dict
    )


def struct(cls: type) -> type:
    """Decorator marking a NamedTuple or dataclass as Spark-schema-compatible."""
    if not is_container(cls):
        raise ValueError(
            f"Only NamedTuple or dataclass instances can be decorated with @struct, not {cls.__name__}"
        )
    return type(
        cls.__name__, cls.__bases__, dict(__pyspark_struct__=..., **cls.__dict__)
    )


def check_pyspark_struct(cls: type) -> None:
    if not isinstance(cls, type):
        raise TypeError(
            f"Expected type, but got instance {cls} of type {type(cls).__name__}"
        )
    if not is_container(cls):
        raise ValueError(f"Type {cls.__name__} can't be used as structure")
    if not is_pyspark_class(cls):
        raise ValueError(f"Looks like type {cls.__name__} missed @struct decorator")


def infer_nullability(typeclass) -> bool:
    return (
        getattr(typeclass, "__origin__", None) is Union
        and NoneType in set(typeclass.__args__)
    )


def unlift_optional(typeclass: Optional[T]) -> T:
    return list(set(typeclass.__args__) - {NoneType})[0]


def maybe_unlift_optional(
    typeclass: Union[T_co, FunctorLike[T_co]],
) -> Tuple[bool, T_co]:
    is_nullable = infer_nullability(typeclass)
    return is_nullable, (unlift_optional(typeclass) if is_nullable else typeclass)


def infer_complex_spark_type(typeclass) -> t.DataType:
    if typeclass.__origin__ in {list, List}:
        (item_T, *_) = typeclass.__args__
        is_nullable, py_type = maybe_unlift_optional(item_T)
        return t.ArrayType(infer_spark_type(py_type), is_nullable)
    if typeclass.__origin__ in {dict, Dict}:
        (k_T, v_T, *_) = typeclass.__args__
        is_nullable_key, py_key_type = maybe_unlift_optional(k_T)
        is_nullable_value, py_value_type = maybe_unlift_optional(v_T)
        if is_nullable_key:
            raise TypeError(
                f"Nullable keys of type {py_key_type} are not allowed in {typeclass}"
            )
        return t.MapType(
            infer_spark_type(py_key_type),
            infer_spark_type(py_value_type),
            is_nullable_value,
        )
    raise TypeError(f"Don't know how to represent {typeclass} in Spark")


def infer_spark_type(typeclass) -> t.DataType:
    """Map a Python typing annotation to its corresponding Spark DataType."""
    if typeclass in (None, NoneType):
        return t.NullType()
    if typeclass is str:
        return t.StringType()
    if typeclass in {bytes, bytearray}:
        return t.BinaryType()
    if typeclass is bool:
        return t.BooleanType()
    if typeclass is date:
        return t.DateType()
    if typeclass is datetime:
        return t.TimestampType()
    if typeclass is Decimal:
        return t.DecimalType(precision=36, scale=6)
    if isinstance(typeclass, type) and issubclass(typeclass, BoundDecimal):
        (precision, scale) = typeclass.__constraints__
        return t.DecimalType(precision=precision, scale=scale)
    if typeclass is float:
        return t.DoubleType()
    if typeclass is int:
        return t.IntegerType()
    if typeclass is long:
        return t.LongType()
    if typeclass is short:
        return t.ShortType()
    if typeclass is byte:
        return t.ByteType()
    if getattr(typeclass, "__origin__", None) is not None:
        return infer_complex_spark_type(typeclass)
    if is_pyspark_class(typeclass):
        return transform(typeclass)
    raise TypeError(f"Don't know how to represent {typeclass} in Spark")


def transform_field(name: str, typeclass: type) -> t.StructField:
    (is_nullable, unwrapped_py_type) = maybe_unlift_optional(typeclass)
    return t.StructField(name, infer_spark_type(unwrapped_py_type), is_nullable)


def transform(typeclass: type) -> t.StructType:
    """Infer a PySpark ``StructType`` from a ``@struct``-marked class.

    The class must be a NamedTuple or dataclass with type-annotated fields.
    """
    check_pyspark_struct(typeclass)
    return t.StructType(
        [transform_field(name, cls) for name, cls in typeclass.__annotations__.items()]
    )
