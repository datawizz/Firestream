import org.scalatest.FunSuite
import org.apache.spark.sql.{DataFrame, SparkSession, SQLContext}
import org.apache.spark.sql.execution.streaming.{LongOffset, MemoryStream}
import org.apache.spark.sql.streaming.StreamingQuery
import java.time.Instant
import java.util.UUID

class SparkConfigTest extends FunSuite {

  val spark = SparkConfig.spark
  implicit val sqlContext: SQLContext = spark.sqlContext
  import spark.implicits._

  // Create a test DataFrame
  val events = MemoryStream[Measurement]
  val testData = Seq(
    Measurement(1, "device1", Instant.now(), 10.0f),
    Measurement(2, "device1", Instant.now(), 20.0f),
    Measurement(3, "device1", Instant.now(), 30.0f),
    Measurement(1, "device1", Instant.now(), 10.0f),
    Measurement(2, "device1", Instant.now(), 20.0f),
    Measurement(3, "device1", Instant.now(), 30.0f),
    Measurement(1, "device1", Instant.now(), 10.0f),
    Measurement(2, "device1", Instant.now(), 20.0f),
    Measurement(3, "device1", Instant.now(), 30.0f),
    Measurement(1, "device1", Instant.now(), 10.0f),
    Measurement(2, "device1", Instant.now(), 20.0f),
    Measurement(3, "device1", Instant.now(), 30.0f),
    Measurement(1, "device1", Instant.now(), 10.0f),
    Measurement(2, "device1", Instant.now(), 20.0f),
    Measurement(3, "device1", Instant.now(), 30.0f)
  )
  val currentOffset = events.addData(testData)
  val testDF = events.toDF()

  test("Test readFromKafka and writeToKafka methods") {
    val testTopic = UUID.randomUUID().toString.take(5)
    // Write test DataFrame to Kafka, await writing all events, commit the offset
    val query = SparkConfig.writeToKafka(testDF, testTopic)
    query.processAllAvailable()
    events.commit(currentOffset.asInstanceOf[LongOffset])

    // // Wait for data to be written
    Thread.sleep(5000)

    // // Read from Kafka
    val readFromKafkaDF = SparkConfig
      .readFromKafka(testTopic)
      .writeStream
      .format("console")
      .outputMode("append")
      .start()
      .processAllAvailable()

    //   .toJSON.collect.foreach(println)

    // Compare the DataFrames
    // assert(testDF.except(readFromKafkaDF).count() == 0)
  }
}
