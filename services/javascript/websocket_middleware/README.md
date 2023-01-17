# Websocket Middleware

Instead of allowing the client application to connect directly to Kafka the connection is proxied using a Middleware microservice which subscribes to topics and then pushes those messages to a websocket connection. The client Javascript code can then connect to this websocket server and will receive up-to-date data including the last N records for each topic from the built in memory cache.

Implementing middleware in this fashion allows Kafka to run in a demilitarized zone which does not have the same security considerations as if the Kafka cluster was accessible directly by client code.