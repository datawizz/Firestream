import java.time.Instant
import org.json4s._
import org.json4s.jackson.JsonMethods._
import org.apache.kafka.clients.consumer.ConsumerRecord


case class Event(
    timestamp: Instant,
    key: String,
    index: Long,
    device_id: String,
    magnitude: Float,
    direction: Float
)




def processEvent(record: ConsumerRecord[String, String]): Unit = {
    implicit val formats = DefaultFormats
    val json = parse(record.value)
    val event = json.extract[Event]
    println(event)
}


import org.apache.kafka.streams.kstream.{TimeWindows, Windowed, WindowSupplier}
import org.apache.kafka.streams.scala.StreamsBuilder
import org.apache.kafka.streams.scala.Serdes._
import org.apache.kafka.streams.scala.ImplicitConversions._
import org.apache.kafka.streams.scala.kstream.{KStream, KTable}


class EventProcessor {

  def process(inputTopic: String, outputTopic: String)(implicit builder: StreamsBuilder): Unit = {
    val windowSize: Long = 5L // Size of the window in number of events
    val windowSupplier: WindowSupplier[String, String, TimeWindows] = TimeWindows.of(windowSize)

    val inputStream: KStream[String, String] = builder.stream[String, String](inputTopic)
    val windowedTable: KTable[Windowed[String], List[String]] = inputStream
      .groupByKey
      .windowedBy(windowSupplier)
      .aggregate(List[String]())((key, value, aggregate) => value :: aggregate, (key, value, aggregate) => value ::: aggregate)

    val outputStream: KStream[String, List[String]] = windowedTable.toStream
    outputStream.to(outputTopic)
  }
}
