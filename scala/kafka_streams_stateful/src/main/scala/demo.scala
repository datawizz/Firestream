package content

import io.circe.generic.auto._
import io.circe.parser._
import io.circe.syntax._
import io.circe.{Decoder, Encoder}
import org.apache.kafka.common.serialization.Serde
import org.apache.kafka.streams.kstream.{
  GlobalKTable,
  JoinWindows,
  TimeWindows,
  Windowed
}
import org.apache.kafka.streams.scala.ImplicitConversions._
import org.apache.kafka.streams.scala._
import org.apache.kafka.streams.scala.kstream.{KGroupedStream, KStream, KTable}
import org.apache.kafka.streams.scala.serialization.Serdes
import org.apache.kafka.streams.scala.serialization.Serdes._
import org.apache.kafka.streams.{KafkaStreams, StreamsConfig, Topology}

import java.time.Duration
import java.time.temporal.ChronoUnit
import java.util.Properties
import scala.concurrent.duration._

object KafkaStreams {

  object Domain {
    type UserId = String
    type Profile = String
    type Product = String
    type OrderId = String
    type Status = String

    case class Order(
        orderId: OrderId,
        userId: UserId,
        products: List[Product],
        amount: Double
    )
    case class Discount(profile: Profile, amount: Double)
    case class Payment(orderId: OrderId, status: Status)

  }

  object Topics {
    val OrdersByUser = "orders-by-user"
    val DiscountProfilesByUser = "discount-profiles-by-user"
    val Discounts = "discounts"
    val Orders = "orders"
    val Payments = "payments"
    val PaidOrders = "paid-orders"
  }

  // source = emits elements
  // flow = transforms elements along the way (e.g. map)
  // sink = ingests elements

  def main(args: Array[String]): Unit = {}
}
