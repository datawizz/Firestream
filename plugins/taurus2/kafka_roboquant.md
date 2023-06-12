First, you need to add the Kafka client dependency to your `build.gradle` or `pom.xml` file, if you haven't done so already.

For Gradle:

```
groovydependencies {
    implementation 'org.apache.kafka:kafka-clients:2.8.1'
}

```

For Maven:

```
xml<dependencies>
    <dependency>
        <groupId>org.apache.kafka</groupId>
        <artifactId>kafka-clients</artifactId>
        <version>2.8.1</version>
    </dependency>
</dependencies>

```

Next, we'll extend your `LiveFeed` class to support reading from a Kafka topic. This code assumes that your Kafka records contain serialized `Event` instances. You might need to modify it to match your actual Kafka configuration and record format.

```
kotlinpackage org.roboquant.feeds

import org.apache.kafka.clients.consumer.ConsumerConfig
import org.apache.kafka.clients.consumer.KafkaConsumer
import org.apache.kafka.common.serialization.StringDeserializer
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.util.Properties

class KafkaLiveFeed(
    private val topic: String,
    private val bootstrapServers: String,
    heartbeatInterval: Long = 10_000
) : LiveFeed(heartbeatInterval) {

    private val props = Properties().apply {
        put(ConsumerConfig.BOOTSTRAP_SERVERS_CONFIG, bootstrapServers)
        put(ConsumerConfig.KEY_DESERIALIZER_CLASS_CONFIG, StringDeserializer::class.java)
        put(ConsumerConfig.VALUE_DESERIALIZER_CLASS_CONFIG, EventDeserializer::class.java)
        put(ConsumerConfig.GROUP_ID_CONFIG, "live-feed-group")
        put(ConsumerConfig.AUTO_OFFSET_RESET_CONFIG, "earliest")
    }

    private val consumer = KafkaConsumer<String, Event>(props)

    init {
        consumer.subscribe(listOf(topic))
    }

    override suspend fun play(channel: EventChannel) {
        this.channel = channel
        try {
            while (isActive) {
                val records = withContext(Dispatchers.IO) { consumer.poll(Duration.ofMillis(heartbeatInterval)) }
                for (record in records) {
                    channel.send(record.value())
                }
            }
        } catch (_: ClosedSendChannelException) {
            // Expected exception
        } finally {
            this.channel = null
            consumer.close()
        }
    }

    private class EventDeserializer : Deserializer<Event> {
        override fun deserialize(topic: String, data: ByteArray): Event {
            // TODO: Implement this method to deserialize your Event from a ByteArray
            throw NotImplementedError("Event deserialization is not implemented yet.")
        }
    }
}

```

This implementation creates a Kafka consumer that reads records from the specified topic. The `play` method polls for new records every `heartbeatInterval` milliseconds. Each record's value (which should be an `Event`) is sent to the `EventChannel`. The Kafka consumer is closed when the feed is no longer active.

Please note that you need to implement the `EventDeserializer` to deserialize your `Event` from a `ByteArray`. This implementation may vary depending on how your `Event` is serialized in Kafka records.

The `KafkaLiveFeed` class takes a Kafka topic and a list of bootstrap servers as arguments. The list of bootstrap servers should be in the form `host1:port1,host2:port2,...`. You can also set other Kafka consumer configuration properties in the `props` variable as necessary for your use case.
