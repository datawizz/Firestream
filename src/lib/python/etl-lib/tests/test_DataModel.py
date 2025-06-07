

"""
Spark type to Iceberg type
This type conversion table describes how Spark types are converted to the Iceberg types. The conversion applies on both creating Iceberg table and writing to Iceberg table via Spark.

Spark	Iceberg	Notes
boolean	boolean	
short	integer	
byte	integer	
integer	integer	
long	long	
float	float	
double	double	
date	date	
timestamp	timestamp with timezone	
char	string	
varchar	string	
string	string	
binary	binary	
decimal	decimal	
struct	struct	
array	list	
map	map

Iceberg type to Spark type
This type conversion table describes how Iceberg types are converted to the Spark types. The conversion applies on reading from Iceberg table via Spark.

Iceberg	Spark	Note
boolean	boolean	
integer	integer	
long	long	
float	float	
double	double	
date	date	
time		Not supported
timestamp with timezone	timestamp	
timestamp without timezone		Not supported
string	string	
uuid	string	
fixed	binary	
binary	binary	
decimal	decimal	
struct	struct	
list	array	
map	map


"""

"""
Supported types for Avro -> Spark SQL conversion
Currently Spark supports reading all primitive types and complex types under records of Avro.

Avro type	Spark SQL type
boolean	BooleanType
int	IntegerType
long	LongType
float	FloatType
double	DoubleType
string	StringType
enum	StringType
fixed	BinaryType
bytes	BinaryType
record	StructType
array	ArrayType
map	MapType

"""


"""
Supported types for Avro -> Spark SQL conversion
Currently Spark supports reading all primitive types and complex types under records of Avro.

Avro type	Spark SQL type
boolean	BooleanType
int	IntegerType
long	LongType
float	FloatType
double	DoubleType
string	StringType
enum	StringType
fixed	BinaryType
bytes	BinaryType
record	StructType
array	ArrayType
map	MapType
union	See below
In addition to the types listed above, it supports reading union types. The following three types are considered basic union types:

union(int, long) will be mapped to LongType.
union(float, double) will be mapped to DoubleType.
union(something, null), where something is any supported Avro type. This will be mapped to the same Spark SQL type as that of something, with nullable set to true. All other union types are considered complex. They will be mapped to StructType where field names are member0, member1, etc., in accordance with members of the union. This is consistent with the behavior when converting between Avro and Parquet.
It also supports reading the following Avro logical types:

Avro logical type	Avro type	Spark SQL type
date	int	DateType
timestamp-millis	long	TimestampType
timestamp-micros	long	TimestampType
decimal	fixed	DecimalType
decimal	bytes	DecimalType
At the moment, it ignores docs, aliases and other properties present in the Avro file.

Supported types for Spark SQL -> Avro conversion
Spark supports writing of all Spark SQL types into Avro. For most types, the mapping from Spark types to Avro types is straightforward (e.g. IntegerType gets converted to int); however, there are a few special cases which are listed below:

Spark SQL type	Avro type	Avro logical type
ByteType	int	
ShortType	int	
BinaryType	bytes	
DateType	int	date
TimestampType	long	timestamp-micros
DecimalType	fixed	decimal
You can also specify the whole output Avro schema with the option avroSchema, so that Spark SQL types can be converted into other Avro types. The following conversions are not applied by default and require user specified Avro schema:

Spark SQL type	Avro type	Avro logical type
BinaryType	fixed	
StringType	enum	
TimestampType	long	timestamp-millis
DecimalType	bytes	decimal


"""

# from enum import Enum


# class SupportedDialects(Enum):
#     """
#     Define the systems where a compatible schema is supported by this library
#     """

#     AVRO = "AVRO"
#     SPARK = "SPARK"
#     JSON = "JSON"
#     SOLR = "SOLR"
#     PETASTORM = "PETASTORM"
#     DELTA = "DELTA"
#     DICTIONARY = "DICTIONARY"

# class AvroTypes(Enum):
#     """
#     Define the Avro to Python types mapping
#     """
#     #TODO: Implement the Avro types


# class DataModel:
#     """
#     Base class for all data models
#     """
#     @classmethod
#     def schema(cls, schema_type: SupportedDialects = SupportedDialects.DICTIONARY):
#         schema = {}
#         for k, v in cls.__annotations__.items():
#             if hasattr(v, '__annotations__'):
#                 schema[k] = v.schema()
#             else:
#                 schema[k] = v.__name__
#         return schema

#     @classmethod
#     def _schema_avro(cls):
#         schema_dict = cls.schema()
#         #TODO: Implement the Avro schema
#         return schema_dict
        
    
#     @classmethod
#     def dependencies(cls):
#         dependencies = []
#         for k, v in cls.__annotations__.items():
#             if hasattr(v, '__annotations__'):
#                 dependencies.append(v)
#         return dependencies


# class MyClass(DataModel):
#     attribute_1: int
#     attribute_2: str


# class MyChildClass(MyClass):
#     attribute_1: MyClass
#     attribute_2: str


# print(MyClass.schema())
# print(MyChildClass.schema())
# print(MyChildClass.schema("AVRO"))
# print(MyChildClass.schema("PARQUET"))
# print(MyChildClass.schema("JSON"))
# print(MyChildClass.schema("SPARK"))
# print(MyChildClass.schema("ICEBERG"))

from enum import Enum

class SchemaDialect(Enum):
    """
    Define the systems where a compatible schema is supported by this library
    """
    JSON = "JSON"
    DICT = "DICT" # the base python dictionary which supports recursive object nesting

class JSONTypes(Enum):
    """
    Define the JSON SCHEMA to Python types mapping
    """
    INT = "integer"
    STR = "string"
    BOOL = "boolean"
    FLOAT = "number"
    DICT = "object"
    LIST = "array"
    # Add more types as per your requirement.


    
class DataModel:
    """
    Base class for all data models
    """
    @classmethod
    def schema(cls, schema_type: SchemaDialect = SchemaDialect.DICT):
        if schema_type == SchemaDialect.JSON:
            return cls._schema_json()
        else:
            return cls._schema_dict()

    @classmethod
    def _schema_dict(cls):
        schema = {}
        for k, v in cls.__annotations__.items():
            if issubclass(v, DataModel):
                schema[k] = v._schema_dict()
            else:
                schema[k] = v.__name__
        return schema

    @classmethod
    def _schema_json(cls):
        schema_dict = cls._schema_dict()
        schema_json = {"type": "object", "properties": {}}

        for k, v in schema_dict.items():
            if isinstance(v, dict):
                schema_json["properties"][k] = v._schema_json()   # Change made here
            else:
                schema_json["properties"][k] = {"type": JSONTypes[v].value}
        return schema_json

    @classmethod
    def dependencies(cls):
        dependencies = []
        for k, v in cls.__annotations__.items():
            if hasattr(v, '__annotations__'):
                dependencies.append(v)
        return dependencies


class MyClass(DataModel):
    attribute_1: int
    attribute_2: str


class MyChildClass(MyClass):
    attribute_3: MyClass
    attribute_4: str


print(MyClass.schema())
print(MyChildClass.schema())
print(MyChildClass.schema(SchemaDialect.JSON))
