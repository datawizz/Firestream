import java.time.Instant
import java.util.Properties
import java.time.Duration

import org.apache.kafka.common.serialization._
import org.apache.kafka.streams._
import org.apache.kafka.streams.kstream._
import org.json.JSONArray
import org.json4s._
import org.json4s.jackson.Serialization
import org.json4s.jackson.Serialization.{write => jsonWrite}
import org.json4s.jackson.Serialization.read

// The schema of the records consumed from Kafka
case class Event(
    device_id: String,
    event_time: Instant,
    direction: Int,
    magnitude: Float
)
// The schema of the output records written to Kafka
case class CombinedEvents(events: List[Event])

object KafkaStreamsAggregation {
  // Configuration for json4s library
  implicit val formats = DefaultFormats

  val inputTopic: String = "event"
  val outputTopic: String = "combined_event"
  val aggregationTime: Duration = Duration.ofSeconds(1)

  def main(args: Array[String]): Unit = {

    // Define the configuration of the Kafka Streams Application
    val props = new Properties()
    props.put(StreamsConfig.APPLICATION_ID_CONFIG, "kafka-streams-example")
    props.put(StreamsConfig.BOOTSTRAP_SERVERS_CONFIG, "localhost:9092")
    props.put(StreamsConfig.PROCESSING_GUARANTEE_CONFIG, "exactly_once")
    props.put(
      StreamsConfig.DEFAULT_KEY_SERDE_CLASS_CONFIG,
      Serdes.String.getClass
    )
    props.put(
      StreamsConfig.DEFAULT_VALUE_SERDE_CLASS_CONFIG,
      Serdes.String.getClass
    )

    // Define the entry point to the Kafka Streams API
    val builder = new StreamsBuilder
    val events: KStream[String, String] = builder.stream(inputTopic)

    val windowedEvents: KStream[Windowed[String], CombinedEvents] = events
      .map((key, value) => (key, read[Event](value)))
      .groupBy((_, event) => event.device_id)
      .windowedBy(TimeWindows.of(aggregationTime))
      .aggregate(
        () => CombinedEvents(List()), // Initializer for the aggregation state
        (
            _,
            event,
            state
        ) => state.copy(events = event :: state.events), // Aggregator
        (
            _,
            _,
            state
        ) => state.copy(events = state.events.sortBy(_.event_time)) // Finalizer
      )

    val combinedEvents: KStream[String, CombinedEvents] =
      windowedEvents.map((_, value) => value)

    val streams = new KafkaStreams(builder.build(), props)
    streams.start()
  }
}

// object Main {
//   def main(args: Array[String]): Unit = {
//     println("Hello, World!")
//   }
// }
