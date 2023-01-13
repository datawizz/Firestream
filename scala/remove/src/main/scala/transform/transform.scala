import org.apache.kafka.common.MetricName
import org.apache.kafka.common.metrics.{Metrics, Sensor}
import org.apache.kafka.common.metrics.stats.{Avg, Max}
import org.apache.kafka.streams.KafkaStreams
import org.apache.kafka.streams.kstream.Windowed
import org.apache.kafka.streams.state.QueryableStoreTypes
import org.apache.kafka.streams.state.ReadOnlyKeyValueStore

class PerformanceMonitor(val streams: KafkaStreams) {
  private val metricConfig =
    new MetricConfig().samples(50).timeWindow(10 * 60 * 1000)
  private val metrics: Metrics = new Metrics(metricConfig)
  private val windowStoreSensor: Sensor = metrics.sensor("window-store")

  windowStoreSensor.add(
    new MetricName(
      "window-store-size-avg",
      "stream-metrics",
      "The average size of the window store"
    ),
    new Avg()
  )
  windowStoreSensor.add(
    new MetricName(
      "window-store-size-max",
      "stream-metrics",
      "The maximum size of the window store"
    ),
    new Max()
  )

  windowStoreSensor.record(getWindowStoreSize)

  def getWindowStoreSize: Long = {
    val windowStore: ReadOnlyKeyValueStore[Windowed[String], List[String]] =
      streams.store(
        EventProcessor.WINDOW_STORE,
        QueryableStoreTypes.keyValueStore[Windowed[String], List[String]]
      )
    windowStore.approximateNumEntries()
  }
}
