import java.time.Instant

case class Measurement(
    index: Long,
    device_id: String,
    event_time: Instant,
    magnitude: Float,
)

case class WienerProcessed(
  index: Long,
  device_id: String,
  event_time: Instant,
  measurement: Float
)

class WienerProcess {
  def calculateWienerProcess(measurement: Measurement, previous: Option[Measurement] = None): WienerProcessed = {
    val lastIndex = previous.map(_.index).getOrElse(0L)
    val timeStep = measurement.index - lastIndex
    val currentValue = previous.map(_.measurement).getOrElse(0f) + measurement.magnitude * timeStep
    WienerProcessed(measurement.index, measurement.device_id, measurement.event_time, currentValue)
  }
}

val wp = new WienerProcess
val measurement1 = Measurement(1, "Device1", Instant.now(), 0.5f)
val measurement2 = Measurement(2, "Device1", Instant.now(), -0.2f)
val measurement3 = Measurement(3, "Device1", Instant.now(), 0.1f)
val processed1 = wp.calculateWienerProcess(measurement1)
val processed2 = wp.calculateWienerProcess(measurement2, Some(measurement1))
val processed3 = wp.calculateWienerProcess(measurement3, Some(measurement2))
println(processed1)
println(processed2)
println(processed3)
