import org.scalatest.funsuite.AnyFunSuite
import org.scalatest.matchers.should.Matchers
import WienerProcess._
import java.time.Instant
import java.time.Duration

class WienerProcessTest extends AnyFunSuite with Matchers {

  test("calculate Wiener process with valid inputs") {
    val previousEvent =
      WienerEvent("device1", Instant.ofEpochMilli(1611602406000L), 0f, 0f, 1f)
    val currentEvent =
      WienerEvent("device1", Instant.ofEpochMilli(1611602407000L))
    val expectedOutput =
      WienerEvent("device1", Instant.ofEpochMilli(1611602407000L), 0.1f, 0f, 1f)
    calculateWienerProcess(
      currentEvent,
      Some(previousEvent)
    ) shouldEqual expectedOutput
  }

}
