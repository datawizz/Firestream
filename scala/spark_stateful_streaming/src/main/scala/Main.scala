import org.apache.spark.sql.{DataFrame, SparkSession}
import org.apache.spark.sql.functions._
import org.apache.spark.sql.types._
import org.apache.spark.sql.streaming.{
  GroupState,
  GroupStateTimeout,
  OutputMode
}
import java.time.Instant

object KafkaStatefulStream {

  val InputTopic: String = "spark_prism"
  val OutputTopic: String = "spark_stateful"
  val BOOTSTRAP_SERVERS: String = "kafka.default.svc.cluster.local:9092"

  case class InputSchema(
      device_id: String,
      magnitude: Float,
      direction: Float,
      timestamp: Instant
  )

  val inputSchema = StructType(
    Seq(
      StructField("device_id", StringType, true),
      StructField("magnitude", FloatType, true),
      StructField("direction", FloatType, true),
      StructField("index", FloatType, true)
    )
  )

  def main(args: Array[String]): Unit = {
    val spark = SparkSession.builder
      .appName("KafkaStreamingExample")
      .config(
        "spark.sql.streaming.checkpointLocation",
        ""
      ) // Disable checkpointing for speed
      .getOrCreate()

    import spark.implicits._

    val inputDf = spark.readStream
      .format("kafka")
      .option("kafka.bootstrap.servers", BOOTSTRAP_SERVERS)
      .option("subscribe", InputTopic)
      .load()

    val jsonDF = inputDf.selectExpr("CAST(value as STRING) as json")
    val jsonData = jsonDF.select(from_json(col("json"), inputSchema).as("data"))
    val outputDF = jsonData
      .select("data.*")
      .as[InputSchema]
      .groupByKey(row => row.device_id)
      .flatMapGroupsWithState(OutputMode.Append, GroupStateTimeout.NoTimeout)(
        updateData _
      )

    // Format the payload for Kafka
    val keyValueDf = outputDF
      .selectExpr("device_id AS key", "to_json(struct(*)) AS value")
      .select("key", "value")

    val query = keyValueDf.writeStream
      .outputMode("append")
      .format("kafka")
      .option("kafka.bootstrap.servers", BOOTSTRAP_SERVERS)
      .option("topic", OutputTopic)
      .start()

    query.awaitTermination()

    // val query = keyValueDf.writeStream
    //   .outputMode("append")
    //   .format("console")
    //   .option("trigger", "1 second")
    //   .option("backpressureEnabled", true)
    //   .start()

    // query
    //   .awaitTermination()
  }

  def updateData(
      key: String,
      values: Iterator[InputSchema],
      state: GroupState[Seq[InputSchema]]
  ): Iterator[InputSchema] = {
    val data = values.toSeq
    if (state.hasTimedOut) {
      state.remove()
      Iterator.empty
    } else {
      state.update(data)
      val lastTwoMessages = state.get.takeRight(2)
      Iterator(lastTwoMessages.head, lastTwoMessages.last)
    }
  }

}
