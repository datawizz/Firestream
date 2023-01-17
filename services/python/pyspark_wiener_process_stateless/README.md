# PySpark Wiener Process Stateless

This service defines a stateless transformation of the "metronome" topic using the Spark Continuous Streams API. This processing is fast but is not a true stochastic process as each message is evaluated in isolation from all other messages in the event stream.