import java.time.Instant
import java.time.Duration

import org.apache.spark.sql.streaming.StreamingQuery

import org.apache.spark.sql.{DataFrame, SparkSession}
import org.apache.spark.sql.functions._
import org.apache.spark.sql.types._
import org.apache.spark.sql.catalyst.ScalaReflection
import org.apache.spark.sql.streaming.{
  GroupState,
  GroupStateTimeout,
  OutputMode
}

import org.apache.spark.sql.{Dataset, SparkSession}
import org.apache.spark.sql.streaming.Trigger
import org.apache.spark.sql.functions._

case class Measurement(
    index: Long,
    key: String,
    event_time: Instant,
    magnitude: Float
)

case class WienerProcessed(
    index: Long,
    key: String,
    event_time: Instant,
    measurement: Float
)

// Define the Spark Application
object SparkConfig {
  val IN_TOPIC: String = "metronome"
  val OUT_TOPIC: String = "spark_structured_streaming_stateful"
  val BOOTSTRAP_SERVERS: String = "kafka.default.svc.cluster.local:9092"

  // Create Spark compatible schemas
  val InputSchemaSpark =
    ScalaReflection.schemaFor[Measurement].dataType.asInstanceOf[StructType]
  val OutputSchemaSpark =
    ScalaReflection.schemaFor[WienerProcessed].dataType.asInstanceOf[StructType]

  val spark = SparkSession.builder
    .appName("KafkaStreamingExample")
    .config("spark.sql.streaming.checkpointLocation", "/tmp/spark")
    .master("local[1]")
    .getOrCreate()

  // Data Source
  def readFromKafka(TOPIC: String = IN_TOPIC): DataFrame = {
    spark.readStream
      .format("kafka")
      .option("kafka.bootstrap.servers", BOOTSTRAP_SERVERS)
      .option("subscribe", TOPIC)
      .option("startingOffsets", "earliest")
      .load()
    // .selectExpr("CAST(value as STRING) as json")
    // .select(from_json(col("json"), InputSchemaSpark).as("data"))
  }

  // Data Sink
  def writeToKafka(df: DataFrame, TOPIC: String = OUT_TOPIC): StreamingQuery = {
    df.selectExpr("key", "to_json(struct(*)) AS value")
      .writeStream
      .outputMode("append")
      .format("kafka")
      .option("kafka.bootstrap.servers", BOOTSTRAP_SERVERS)
      .option("topic", TOPIC)
      .start()

  }

}

// // A test case of the above

// class KafkaStreamProcessor {

//   def processStream(): Unit = {
//     val inputDf = SparkConfig.readFromKafka()
//     val outputDf = inputDf
//       .as[Measurement]
//       .map(record => {
//         // perform some operation on the record
//         println(record)
//         record
//       })
//     SparkConfig.writeToKafka(outputDf)
//   }
// }

// // TODO use this
// class WienerProcess {
//   def calculateWienerProcess(
//       measurement: Measurement,
//       previous: Option[Measurement] = None
//   ): WienerProcessed = {
//     val lastIndex = previous.map(_.index).getOrElse(0L)
//     val timeStep = measurement.index - lastIndex
//     val currentValue = previous
//       .map(_.measurement)
//       .getOrElse(0f) + measurement.magnitude * timeStep
//     WienerProcessed(
//       measurement.index,
//       measurement.device_id,
//       measurement.event_time,
//       currentValue
//     )
//   }
// }

// object KafkaStatefulStream {

//   def main(args: Array[String]): Unit = {

//     // Define the user defined function
//     val wienerProcessUDF =
//       udf((measurement: Measurement, previous: Measurement) =>
//         WienerProcess.calculateWienerProcess(measurement, Some(previous))
//       )

//     // Use the user defined function to process the stream
//     val processed = measurements
//       .withWatermark("event_time", "10 seconds")
//       .groupByKey(_.device_id)
//       .mapGroupsWithState[Measurement, WienerProcessed](
//         GroupStateTimeout.ProcessingTimeTimeout
//       ) { (deviceId, measurements, state) =>
//         if (state.hasTimedOut) {
//           state.clear()
//         }
//         var previousMeasurement: Option[Measurement] = None
//         if (state.exists) {
//           previousMeasurement = Some(state.get)
//         }
//         val processedMeasurements = measurements.map(measurement =>
//           WienerProcess.calculateWienerProcess(measurement, previousMeasurement)
//         )
//         state.update(measurements.last)
//         processedMeasurements
//       }

//     val outputDF = jsonData
//       .select("data.*")
//       .as[InputSchema]
//       .groupByKey(row => row.device_id)
//       .flatMapGroupsWithState(OutputMode.Append, GroupStateTimeout.NoTimeout)(
//         updateData _
//       )

//     query.awaitTermination()

//   }

//   // def updateData(
//   //     key: String,
//   //     values: Iterator[InputSchema],
//   //     state: GroupState[(Option[InputSchema], List[InputSchema])]
//   // ): Iterator[StatefulSchema] = {
//   //   val data = values.toList
//   //   val previousRecord = state.getOption.map(_._1).getOrElse(None)
//   //   if (state.hasTimedOut) {
//   //     state.remove()
//   //     Iterator.empty
//   //   } else {
//   //     val currentRecord = data.headOption
//   //     val output = currentRecord
//   //       .map(cr => (previousRecord, cr))
//   //       .filter(_._1.isDefined)
//   //       .map(pair => pair._1.get :: pair._2 :: Nil)
//   //       .getOrElse(Nil)
//   //     state.update((currentRecord, output))
//   //     Iterator(StatefulSchema(key, output))
//   //   }
//   // // }
//   // def updateData(
//   //     key: String,
//   //     values: Iterator[InputSchema],
//   //     state: GroupState[(Option[InputSchema], List[WienerProcess])]
//   // ): Iterator[WienerProcess] = {
//   //   val data = values.toList
//   //   val previousRecord =
//   //     state.getOption.map(_._1).getOrElse(InputSchema(key, Instant.MIN, 0, 0))
//   //   if (state.hasTimedOut) {
//   //     state.remove()
//   //     Iterator.empty
//   //   } else {
//   //     /*
//   //       The basic formula for Brownian motion is:

//   //       dx = μdt + σdW

//   //       Where:

//   //       x is the position of the particle at time t
//   //       μ is the drift coefficient, which represents the average velocity of the particle
//   //       dt is a small time interval
//   //       σ is the diffusion coefficient, which represents the randomness or volatility of the particle's movement
//   //       dW is a random variable that represents the change in the particle's position over the time interval dt.
//   //         It is known as a Wiener process and is typically modeled as a normal distribution with mean 0 and variance dt
//   //      */

//   //     val currentRecord = data.headOption
//   //     val output = currentRecord
//   //       .filter(_.device_id.isDefined)
//   //       .map { case (prev, curr) =>
//   //         val dt =
//   //           (curr.event_time.toEpochMilli - prev.event_time.toEpochMilli) / 1000.0
//   //         val dW = (curr.direction match {
//   //           case 0.0 => -1
//   //           case 1.0 => 1
//   //         }) * curr.magnitude * dt
//   //         WienerProcess(key, startingValue + dW, curr.event_time)
//   //       }
//   //       .toList
//   //     state.update((currentRecord, output))
//   //     output.iterator
//   //   }
//   // }

// }
