# Websocket Middleware

Instead of allowing the client application to connect directly to Kafka the connection is proxied using a Middleware microservice which subscribes to topics and then pushes those messages to a websocket connection. The client Javascript code can then connect to this websocket server and will receive up-to-date data the built in memory cache.

Implementing middleware in this fashion allows Kafka to run in a demilitarized zone which does not have the same security considerations as if the Kafka cluster was accessible directly by client code. This effectively pushes the security concerns to the Middleware, which at the moment, does not have authentication.

# TODO

There are lots of things to do!

0. Implement Role Based Authorization via Grants
1. Add authentication by checking postgres if the provided token is valid.
2. Listen to Kafka for CDC that invalidates a grant to a user (depends on debezium being deployed)
3. Add SQL sanitization via some method (maybe an external API?) but ensure that it is appropriatley scoped before being sent for execution. This includes RBAC!