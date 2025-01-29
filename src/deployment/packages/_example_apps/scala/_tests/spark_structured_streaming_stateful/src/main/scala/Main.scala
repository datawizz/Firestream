import org.apache.spark.sql.{DataFrame, SparkSession}
import org.apache.spark.sql.functions._
import org.apache.spark.sql.types._
import org.apache.spark.sql.catalyst.ScalaReflection
import org.apache.spark.sql.streaming.{
  GroupState,
  GroupStateTimeout,
  OutputMode
}
import java.time.Instant
import java.time.Duration

object KafkaStatefulStream {

  val InputTopic: String = "metronome"
  val OutputTopic: String = "spark_structured_streaming_stateful"
  val BOOTSTRAP_SERVERS: String = "kafka.default.svc.cluster.local:9092"

  // Define the schema of the input data stream
  case class InputSchema(
      device_id: String,
      event_time: Instant,
      magnitude: Float,
      direction: Float
  )

  // Define the starting value of Wiener process
  val startingValue: Double = 0.0

  // Define the schema of the output state
  case class WienerProcess(
      device_id: String,
      measurement: Float,
      event_time: Instant
  )

  // Create Spark compatible schemas
  val InputSchemaSpark =
    ScalaReflection.schemaFor[InputSchema].dataType.asInstanceOf[StructType]
  val OutputSchemaSpark =
    ScalaReflection.schemaFor[WienerProcess].dataType.asInstanceOf[StructType]

  def main(args: Array[String]): Unit = {
    val spark = SparkSession.builder
      .appName("KafkaStreamingExample")
      .config(
        "spark.sql.streaming.checkpointLocation",
        "/tmp/spark"
      )
      .getOrCreate()

    import spark.implicits._

    val inputDf = spark.readStream
      .format("kafka")
      .option("kafka.bootstrap.servers", BOOTSTRAP_SERVERS)
      .option("subscribe", InputTopic)
      .load()

    val jsonDF = inputDf.selectExpr("CAST(value as STRING) as json")
    val jsonData =
      jsonDF.select(from_json(col("json"), InputSchemaSpark).as("data"))
    val outputDF = jsonData
      .select("data.*")
      .as[InputSchema]
      .groupByKey(row => row.device_id)
      .flatMapGroupsWithState(OutputMode.Append, GroupStateTimeout.NoTimeout)(
        updateData _
      )

    // Format the payload for Kafka
    val keyValueDf = outputDF
      .selectExpr("to_json(struct(*)) AS value")
      .select("value")

    val query = keyValueDf.writeStream
      .outputMode("append")
      .format("kafka")
      .option("kafka.bootstrap.servers", BOOTSTRAP_SERVERS)
      .option("topic", OutputTopic)
      .start()

    query.awaitTermination()

  }

  // def updateData(
  //     key: String,
  //     values: Iterator[InputSchema],
  //     state: GroupState[(Option[InputSchema], List[InputSchema])]
  // ): Iterator[StatefulSchema] = {
  //   val data = values.toList
  //   val previousRecord = state.getOption.map(_._1).getOrElse(None)
  //   if (state.hasTimedOut) {
  //     state.remove()
  //     Iterator.empty
  //   } else {
  //     val currentRecord = data.headOption
  //     val output = currentRecord
  //       .map(cr => (previousRecord, cr))
  //       .filter(_._1.isDefined)
  //       .map(pair => pair._1.get :: pair._2 :: Nil)
  //       .getOrElse(Nil)
  //     state.update((currentRecord, output))
  //     Iterator(StatefulSchema(key, output))
  //   }
  // // }
  // def updateData(
  //     key: String,
  //     values: Iterator[InputSchema],
  //     state: GroupState[(Option[InputSchema], List[WienerProcess])]
  // ): Iterator[WienerProcess] = {
  //   val data = values.toList
  //   val previousRecord =
  //     state.getOption.map(_._1).getOrElse(InputSchema(key, Instant.MIN, 0, 0))
  //   if (state.hasTimedOut) {
  //     state.remove()
  //     Iterator.empty
  //   } else {
  //     /*
  //       The basic formula for Brownian motion is:

  //       dx = μdt + σdW

  //       Where:

  //       x is the position of the particle at time t
  //       μ is the drift coefficient, which represents the average velocity of the particle
  //       dt is a small time interval
  //       σ is the diffusion coefficient, which represents the randomness or volatility of the particle's movement
  //       dW is a random variable that represents the change in the particle's position over the time interval dt.
  //         It is known as a Wiener process and is typically modeled as a normal distribution with mean 0 and variance dt
  //      */

  //     val currentRecord = data.headOption
  //     val output = currentRecord
  //       .filter(_.device_id.isDefined)
  //       .map { case (prev, curr) =>
  //         val dt =
  //           (curr.event_time.toEpochMilli - prev.event_time.toEpochMilli) / 1000.0
  //         val dW = (curr.direction match {
  //           case 0.0 => -1
  //           case 1.0 => 1
  //         }) * curr.magnitude * dt
  //         WienerProcess(key, startingValue + dW, curr.event_time)
  //       }
  //       .toList
  //     state.update((currentRecord, output))
  //     output.iterator
  //   }
  // }

  def updateData(
      key: String,
      values: Iterator[InputSchema],
      state: GroupState[(Option[InputSchema], List[WienerProcess])]
  ): Iterator[WienerProcess] = {
    val data = values.toList
    val previousRecord = state.getOption.map(_._1).getOrElse(None)
    if (state.hasTimedOut) {
      state.remove()
      Iterator.empty
    } else {
      val currentRecord = data.headOption
      val output = currentRecord
        .flatMap(cr => previousRecord.map(pr => (pr, cr)))
        .map { case (pr, cr) =>
          val dt =
            (cr.event_time.toEpochMilli - pr.event_time.toEpochMilli) / 1000.0
          val direction = if (cr.direction == 0.0) -1 else 1
          val dW = direction * cr.magnitude * dt
          WienerProcess(
            cr.device_id,
            (startingValue.toDouble + dW).toFloat,
            cr.event_time
          )
        }
      state.update((currentRecord, output.toList))

      output.toList.iterator
    }
  }

}
