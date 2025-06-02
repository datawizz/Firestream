from petastorm.spark import SparkDatasetConverter, make_spark_converter
import tensorflow.compat.v1 as tf  # pylint: disable=import-error

from pyspark.sql import SparkSession as spark

# specify a cache dir first.
# the dir is used to save materialized spark dataframe files
spark.conf.set(
    SparkDatasetConverter.PARENT_CACHE_DIR_URL_CONF,
    "file:/workspace/lib/python/etl_lib/etl_lib/services/spark/example",
)

df = ...  # `df` is a spark dataframe

# create a converter from `df`
# it will materialize `df` to cache dir.
converter = make_spark_converter(df)

# make a tensorflow dataset from `converter`
with converter.make_tf_dataset() as dataset:
    # the `dataset` is `tf.data.Dataset` object
    # dataset transformation can be done if needed
    dataset = dataset.map(...)
    # we can train/evaluate model on the `dataset`
    model.fit(dataset)
    # when exiting the context, the reader of the dataset will be closed

# delete the cached files of the dataframe.
converter.delete()


# from petastorm.spark import SparkDatasetConverter, make_spark_converter

# # specify a cache dir first.
# # the dir is used to save materialized spark dataframe files
# spark.conf.set(SparkDatasetConverter.PARENT_CACHE_DIR_URL_CONF, "hdfs:/...")

# df_train, df_test = ...  # `df_train` and `df_test` are spark dataframes
# model = Net()

# # create a converter_train from `df_train`
# # it will materialize `df_train` to cache dir. (the same for df_test)
# converter_train = make_spark_converter(df_train)
# converter_test = make_spark_converter(df_test)

# # make a pytorch dataloader from `converter_train`
# with converter_train.make_torch_dataloader() as dataloader_train:
#     # the `dataloader_train` is `torch.utils.data.DataLoader` object
#     # we can train model using the `dataloader_train`
#     train(model, dataloader_train, ...)
#     # when exiting the context, the reader of the dataset will be closed

# # the same for `converter_test`
# with converter_test.make_torch_dataloader() as dataloader_test:
#     test(model, dataloader_test, ...)

# # delete the cached files of the dataframes.
# converter_train.delete()
# converter_test.delete()
