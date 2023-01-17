import org.apache.kafka.streams.kstream.TimeWindows
import org.apache.kafka.streams.kstream.Windowed
import org.apache.kafka.streams.kstream.TimeWindows
import org.apache.kafka.streams.StreamsBuilder
import org.apache.kafka.streams.Serdes
import org.apache.kafka.streams.ImplicitConversions._
import org.apache.kafka.streams.kstream.{KStream, KTable}

case class Event(event_time: Instant, direction: Int, magnitude: Float)

class EventProcessor {

  def process(inputTopic: String, outputTopic: String)(implicit
      builder: StreamsBuilder
  ): Unit = {
    val windowSize: Long = 5 // Size of the window in number of events
    val windowSupplier: WindowSupplier[String, String, TimeWindows] =
      TimeWindows.of(windowSize)

    val inputStream: KStream[String, String] =
      builder.stream[String, String](inputTopic)
    val windowedTable: KTable[Windowed[String], List[String]] =
      inputStream.groupByKey
        .windowedBy(windowSupplier)
        .aggregate(List[String]())(
          (key, value, aggregate) => value :: aggregate,
          (key, value, aggregate) => value ::: aggregate
        )

    val outputStream: KStream[String, List[String]] = windowedTable.toStream
    outputStream.to(outputTopic)
  }
}
