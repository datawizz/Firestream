import pyspark.sql.types as pyspark_type

"""
Use Pydantic to define a set of acceptable types
"""


from pydantic import BaseModel as PydanticBaseModel
from pydantic import ValidationError
from pydantic.fields import ModelField
from typing import TypeVar, Generic

AgedType = TypeVar("AgedType")
QualityType = TypeVar("QualityType")


# # This is not a pydantic model, it's an arbitrary generic class
# class TastingModel(Generic[AgedType, QualityType]):
#     def __init__(self, name: str, aged: AgedType, quality: QualityType):
#         self.name = name
#         self.aged = aged
#         self.quality = quality

#     @classmethod
#     def __get_validators__(cls):
#         yield cls.validate

#     @classmethod
#     # You don't need to add the "ModelField", but it will help your
#     # editor give you completion and catch errors
#     def validate(cls, v, field: ModelField):
#         if not isinstance(v, cls):
#             # The value is not even a TastingModel
#             raise TypeError("Invalid value")
#         if not field.sub_fields:
#             # Generic parameters were not provided so we don't try to validate
#             # them and just return the value as is
#             return v
#         aged_f = field.sub_fields[0]
#         quality_f = field.sub_fields[1]
#         errors = []
#         # Here we don't need the validated value, but we want the errors
#         valid_value, error = aged_f.validate(v.aged, {}, loc="aged")
#         if error:
#             errors.append(error)
#         # Here we don't need the validated value, but we want the errors
#         valid_value, error = quality_f.validate(v.quality, {}, loc="quality")
#         if error:
#             errors.append(error)
#         if errors:
#             raise ValidationError(errors, cls)
#         # Validation passed without errors, return the same instance received
#         return v


# class Model:
#     # for wine, "aged" is an int with years, "quality" is a float
#     wine: TastingModel[int, float]
#     # for cheese, "aged" is a bool, "quality" is a str
#     cheese: TastingModel[bool, str]
#     # for thing, "aged" is a Any, "quality" is Any
#     thing: TastingModel


# model = Model(
#     # This wine was aged for 20 years and has a quality of 85.6
#     wine=TastingModel(name="Cabernet Sauvignon", aged=20, quality=85.6),
#     # This cheese is aged (is mature) and has "Good" quality
#     cheese=TastingModel(name="Gouda", aged=True, quality="Good"),
#     # This Python thing has aged "Not much" and has a quality "Awesome"
#     thing=TastingModel(name="Python", aged="Not much", quality="Awesome"),
# )


from typing import List, Union, Dict, Optional
from enum import Enum
from etl_lib import DataModel


class DataField(PydanticBaseModel):
    """
    The base field object

    partition_by:
        If this field should be used when writing to persistent storage
    """

    def __init__(self, partition_by: Union[bool, None], alias: Optional[str]):
        pass

        class ValidationError(DataModel):
            """
            A instance of a invalidation error of a record.
            Emitted when the __validation__ function over the context
            Only meant to be applied when time is not of the essence
            """

            pass

        return  # reference to "field" similar to how dataclasses.field works


class UUID_Type(DataModel):
    """
    The Universal Unique Identifier is a hash over a string or concatonation of fields / columns
    """

    pass


class Float_Type(DataModel):
    """
    A fixed length floating point number
    """

    def __init__(self, partition_by: Union[bool, None], alias: Optional[str]):
        super().__init__(partition_by, alias)


class time_units(Enum):
    """
    Supported units of measure of time
    """

    SECOND = 1
    DAY = 2


class Timestamp_Type(DataModel):
    """
    The Timestamp Type is a exact point in time


    time_units:
        An enum of the units of time that should be expected in this field

    """

    def __init__(
        self,
        partition_by: Union[bool, None],
        alias: Optional[str],
        time_unit: Optional[time_units] = None,
    ):
        super().__init__(partition_by, alias)


class TimeDelta_Type(DataModel):
    """
    Represents a precise amount of time between two timestamps
    The start - end
    """

    pass


class StochasticFloat_Type:
    """
    The Stochastic Float is a fixed length floating point integer that progresses
    as a stochastic process via Brownian Motion at fixed time intervals

    standard_deviation:
        The expected standard deviation over a large enough sample.
        This is used as a direct input when generating data

    mean:
        The expected average value of this field over a large enough sample

    distribution:
        A sampleable probability distribution
        defaults to a normal probability
        Method to sample in numpy and pyro?

    """

    def __init__(
        self, standard_deviation: int, mean: float, skew: float, kurtosis: float
    ):
        pass


class Date_Type:
    """
    A Date, not a datetime. Useful for partitioning and filtering
    """

    pass


class EventTime_Type(DataModel):
    """
    The EventTime_Type demarks a field which carries a timestamp which represents the precise
    point in time that the event happened.

    EventTimes can be sparadic and therefore when generating they are created through a stochastic process

    Each DataModel can contain at most one EventTime typed field.

    The EventTime field is used to join streams together.

    intervals:
        A list of intervals which are expected to be present in this field
        Determined at runtime based on passed variable

    """

    def __init__(self):
        pass


# class DictType:
#     """
#     Implements a hash map

#     """

#     _types_table = (
#         (np.int8, pyspark_type.ByteType()),
#         (np.uint8, pyspark_type.ShortType()),
#         (np.int16, pyspark_type.ShortType()),
#         (np.uint16, pyspark_type.IntegerType()),
#         (np.int32, pyspark_type.IntegerType()),
#         (np.int64, pyspark_type.LongType()),
#         (np.float32, pyspark_type.FloatType()),
#         (np.float64, pyspark_type.DoubleType()),
#         (np.string_, pyspark_type.StringType()),
#         (np.str_, pyspark_type.StringType()),
#         (np.unicode_, pyspark_type.StringType()),
#         (np.bool_, pyspark_type.BooleanType()),
#         (np.array, "sets a array of array of a type, recursively"),
#     )


# def _numpy_and_codec_from_arrow_type(field_type):
#     from pyarrow import types

#     if types.is_int8(field_type):
#         np_type = np.int8
#     elif types.is_uint8(field_type):
#         np_type = np.uint8
#     elif types.is_int16(field_type):
#         np_type = np.int16
#     elif types.is_int32(field_type):
#         np_type = np.int32
#     elif types.is_int64(field_type):
#         np_type = np.int64
#     elif types.is_string(field_type):
#         np_type = np.unicode_
#     elif types.is_boolean(field_type):
#         np_type = np.bool_
#     elif types.is_float32(field_type):
#         np_type = np.float32
#     elif types.is_float64(field_type):
#         np_type = np.float64
#     elif types.is_decimal(field_type):
#         np_type = Decimal
#     elif types.is_binary(field_type):
#         np_type = np.string_
#     elif types.is_fixed_size_binary(field_type):
#         np_type = np.string_
#     elif types.is_date(field_type):
#         np_type = np.datetime64
#     elif types.is_timestamp(field_type):
#         np_type = np.datetime64
#     elif types.is_list(field_type):
#         np_type = _numpy_and_codec_from_arrow_type(field_type.value_type)
#     else:
#         raise ValueError(
#             "Cannot auto-create unischema due to unsupported column type {}".format(
#                 field_type
#             )
#         )
#     return np_type


class String_Type:
    pass


class Array_Type:
    """
    Implements a array or list of ordered items
    """

    def __class_getitem__(cls, key):
        """
        Handles subtypes
        https://docs.python.org/3/reference/datamodel.html#emulating-generic-types
        """
        pass


class Map_Type:

    """
    Implements a hash map of <key:value>

    key and value may be any other type
    """

    def __class_getitem__(cls, key):
        """
        Handles subtypes
        https://docs.python.org/3/reference/datamodel.html#emulating-generic-types
        """
        pass
