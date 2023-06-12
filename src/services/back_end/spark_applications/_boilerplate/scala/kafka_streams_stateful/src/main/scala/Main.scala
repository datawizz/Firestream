// import java.time.Instant
// import java.util.Properties
// import java.time.Duration

// import org.apache.kafka.common.serialization._
// import org.apache.kafka.streams._
// import org.apache.kafka.streams.kstream._
// import org.json.JSONArray
// import org.json4s._
// import org.json4s.jackson.Serialization
// import org.json4s.jackson.Serialization.{write => jsonWrite}
// import org.json4s.jackson.Serialization.read

// // The schema of the records consumed from Kafka
// case class Event(
//     device_id: String,
//     event_time: Instant,
//     direction: Int,
//     magnitude: Float
// )
// // The schema of the output records written to Kafka
// case class CombinedEvents(events: List[Event])

// object KafkaStreamsAggregation {
//   // Configuration for json4s library
//   implicit val formats = DefaultFormats

//   val inputTopic: String = "event"
//   val outputTopic: String = "combined_event"
//   val aggregationTime: Duration = Duration.ofSeconds(1)

//   def main(args: Array[String]): Unit = {

//     // Define the configuration of the Kafka Streams Application
//     val props = new Properties()
//     props.put(StreamsConfig.APPLICATION_ID_CONFIG, "kafka-streams-example")
//     props.put(StreamsConfig.BOOTSTRAP_SERVERS_CONFIG, "localhost:9092")
//     props.put(StreamsConfig.PROCESSING_GUARANTEE_CONFIG, "exactly_once")
//     props.put(
//       StreamsConfig.DEFAULT_KEY_SERDE_CLASS_CONFIG,
//       Serdes.String.getClass
//     )
//     props.put(
//       StreamsConfig.DEFAULT_VALUE_SERDE_CLASS_CONFIG,
//       Serdes.String.getClass
//     )

//     // Define the entry point to the Kafka Streams API
//     val builder = new StreamsBuilder
//     val events: KStream[String, String] = builder.stream(inputTopic)

//     val windowedEvents: KStream[Windowed[String], CombinedEvents] = events
//       .map((key, value) => (key, read[Event](value)))
//       .groupBy((_, event) => event.device_id)
//       .windowedBy(TimeWindows.of(aggregationTime))
//       .aggregate(
//         () => CombinedEvents(List()), // Initializer for the aggregation state
//         (
//             _,
//             event,
//             state
//         ) => state.copy(events = event :: state.events), // Aggregator
//         (
//             _,
//             _,
//             state
//         ) => state.copy(events = state.events.sortBy(_.event_time)) // Finalizer
//       )

//     val combinedEvents: KStream[String, CombinedEvents] =
//       windowedEvents.map((_, value) => value)

//     val streams = new KafkaStreams(builder.build(), props)
//     streams.start()
//   }
// }

// object Main {
//   def main(args: Array[String]): Unit = {
//     println("Hello, World!")
//   }
// }

import java.time.Instant
import scala.reflect.runtime.universe._
import org.apache.kafka.common.serialization._
import org.apache.kafka.streams._
import org.apache.kafka.streams.kstream._
import org.apache.kafka.streams.state._
import org.apache.kafka.streams.scala.ImplicitConversions._
import org.apache.kafka.streams.scala._
import org.apache.kafka.streams.scala.kstream._
import org.apache.kafka.streams.scala.Serdes._

// Define the Kafka configurations
object StreamsConfig {
  val props = new Properties()
   props.put(StreamsConfig.APPLICATION_ID_CONFIG, "kafka-streams-stateful")
  props.put(StreamsConfig.BOOTSTRAP_SERVERS_CONFIG, "kafka.default.svc.cluster.local:9092")
  props.put(StreamsConfig.PROCESSING_GUARANTEE_CONFIG, StreamsConfig.EXACTLY_ONCE)
  props.put(StreamsConfig.DEFAULT_KEY_SERDE_CLASS_CONFIG, Serdes.String.getClass)
  props.put(StreamsConfig.DEFAULT_VALUE_SERDE_CLASS_CONFIG, Serdes.String.getClass)
}

 
// Define the input and output schemas
case class InSchema(key: String, event_time: Instant, magnitude: Float, direction: Float, index: Int)
case class OutSchema(key: String, list: List[InSchema])

object LowLatencyAggregation {
    def main(args: Array[String]): Unit = {
        val builder = new StreamsBuilder
        val inputTopic = "input-topic"
        val outputTopic = "output-topic"
        val storeName = ... // store name

        // Use the properties from the StreamsConfig object
        val streams = new KafkaStreams(builder.build, StreamsConfig.props)


        val inputStream: KStream[String, InSchema] = builder.stream(inputTopic, Consumed.with(Serdes.String, caseClassSerde[InSchema]))
        val groupedStream: KGroupedStream[String, InSchema] = inputStream.groupByKey
        val aggregatedStream: KTable[String, OutSchema] = groupedStream.aggregate(
        () => OutSchema("", List()), // Initializer function
        (key: String, value: InSchema, aggregate: OutSchema) => {
        // Aggregation function that pair each message with the most recent message
        OutSchema(key, value :: aggregate.list)
        },
        Materialized.asString, OutSchema, MemoryStore
        .withLoggingDisabled()
        .withCachingEnabled()
        .withExpireAfterAccess(Duration.ofSeconds(5))
        )
        val outputStream = aggregatedStream.toStream
        outputStream.to(outputTopic, Produced.with(Serdes.String, caseClassSerde[OutSchema]))
        streams.start()
    }

    def caseClassSerde[T]()(implicit t: TypeTag[T]): Serde[T] = {
        val serde = new JsonSerializer[T]()
        Serdes.serdeFrom(serde, serde)
    }
}
