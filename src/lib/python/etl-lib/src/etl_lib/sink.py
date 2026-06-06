import abc

from etl_lib.context import DataContext


class DataSink(DataContext, metaclass=abc.ABCMeta):
    """Abstract base for writing data."""

    def __init__(self, **kwargs) -> None:
        super().__init__(**kwargs)

    @abc.abstractmethod
    def sink(self) -> None:
        """Write out the DataFrame held in ``self.spark_df``."""
        ...


class Console_DataSink(DataSink):
    """Write the DataFrame and its schema to STDOUT.

    Useful for development and tests. Streaming output is not yet supported.
    """

    def sink(self) -> None:
        if self.streaming:
            raise NotImplementedError(
                "Console_DataSink does not yet support streaming DataFrames"
            )
        self.spark_df.show()
        self.spark_df.printSchema()


class S3_DataSink(DataSink):
    """Write a Spark DataFrame to S3, partitioned by the configured fields.

    The write path is not yet implemented — partitioning, path layout, and
    catalog-aware write semantics still need to be defined.
    """

    def sink(self) -> None:
        raise NotImplementedError("S3_DataSink.sink is not yet implemented")
