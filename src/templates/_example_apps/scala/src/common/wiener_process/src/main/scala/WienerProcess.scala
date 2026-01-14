import java.time.Instant
import java.time.Duration

case class WienerEvent(
    device_id: String,
    event_time: Instant,
    measurement: Float = 0,
    mean: Float = 0,
    standard_deviation: Float = 1
)

object WienerProcess {

  def main(args: Array[String]): Unit = {}

  def calculateWienerProcess(
      currentEvent: WienerEvent,
      previousEvent: Option[WienerEvent]
  ): WienerEvent = {
    val dt = currentEvent.event_time.toEpochMilli - previousEvent
      .map(_.event_time.toEpochMilli)
      .getOrElse(currentEvent.event_time.toEpochMilli)

    val dw = scala.util.Random.nextGaussian() * math.sqrt(
      dt
    ) * currentEvent.standard_deviation
    val measurement = previousEvent
      .map(_.measurement)
      .getOrElse(0.0f) + currentEvent.mean * dt + dw
    currentEvent.copy(measurement = measurement.toFloat)
  }

}
