
const { NodeSDK } = require('@opentelemetry/sdk-node');
const { OTLPTraceExporter } = require('@opentelemetry/exporter-trace-otlp-proto');

const sdk = new NodeSDK({
    traceExporter: new OTLPTraceExporter({
        url: 'signoz-otel-collector.default.svc.cluster.local:4317', // specify the collector's grpc endpoint
    }),
});
sdk.start()
const { context, trace, SpanKind, diag } = require('@opentelemetry/api');


const WebSocket = require('ws');
const express = require('express');
const http = require('http');
const Kafka = require('node-rdkafka');



const direction = process.env.DIRECTION || "FOLLOW";
const debug = false;
const bootstrapServers = 'kafka.default.svc.cluster.local:9092';
const topics = ['A_RUSTY_TOPIC'];
const topicArrays = {};
const app_name = "websocket-middleware"
const tracer = trace.getTracer(app_name);


// Health Checks Server
const app = express();
const server = http.createServer(app);
app.get('/health', (req, res) => {
    const span = tracer.startSpan('health-check');
    res.send('OK');
    span.end();
});
server.listen(8000);

let messageCounter = 0;
// Print out message count every 5 seconds
setInterval(() => {
    console.log(`Messages processed in the last 5 seconds: ${messageCounter}`);
    messageCounter = 0;
}, 5000);


function lead_role() {
    // Code to read from Kafka and publish to WebSocket
    const wss = new WebSocket.Server({ port: 8080 });
    const clients = [];
    topics.forEach(topic => {
        topicArrays[topic] = [];
    });

    const connectToKafka = (consumer) => {
        const span = tracer.startSpan('connectToKafka', {
            kind: SpanKind.CLIENT,
        });
        consumer.connect();

        consumer
            .on('ready', () => {
                span.end();
                consumer.subscribe(topics);
                consumer.consume();
            })
            .on('data', (data) => {
                messageCounter++;
                topicArrays[data.topic].push(JSON.parse(data.value));
                if (debug) {
                    console.log(data.topic);
                    console.log(topicArrays[data.topic].length);
                    console.log(data.value);
                }
                if (topicArrays[data.topic].length > 100) {
                    topicArrays[data.topic].pop();
                }
            })
            .on('event.error', (err) => {
                // TODO Another span here?
                console.error('Error in Kafka consumer:', err);
                console.log('Reconnecting...');
                setTimeout(() => connectToKafka(consumer), 5000);
            });
    };

    const consumer = new Kafka.KafkaConsumer({
        'group.id': 'WebSocketMiddleware2',
        'metadata.broker.list': bootstrapServers,
        'socket.keepalive.enable': true,
        'enable.auto.commit': false,
        'reconnect.backoff.jitter.ms': 200,
        'reconnect.backoff.ms': 200,
        'reconnect.backoff.max.ms': 2000,
    }, {});

    wss.on('connection', (ws) => {
        const wsSpan = tracer.startSpan('WebSocket Connection', {
            kind: SpanKind.CLIENT,
        });
        clients.push({ ws, connected: true, lastPong: new Date() });

        setInterval(() => {
            topics.forEach(topic => {
                while (topicArrays[topic].length > 0) {
                    const _message = topicArrays[topic].shift();
                    clients.forEach(client => {
                        if (client.connected) client.ws.send(JSON.stringify(_message).toString());
                    });
                }
            });
        }, 100);

        // send a ping message every 5 seconds
        setInterval(() => {
            clients.forEach(client => {
                if (client.connected) {
                    client.ws.ping();
                    if (debug) {
                        console.log('Ping sent to client: ', clients.find(client => client.ws === ws));
                    }
                }
            });
        }, 5000);

        // listen for pong responses from the client and update state to lastPong instant
        ws.on('pong', () => {
            clients.forEach(client => {
                if (client.ws === ws) {
                    client.lastPong = new Date();
                }
            });
        });

        // check every 10 seconds if client is still connected
        setInterval(() => {
            clients.forEach(client => {
                if (client.ws === ws && (new Date() - client.lastPong) > 10000) {
                    client.connected = false;
                    console.log('Client disconnected due to inactivity');
                }
            });
        }, 10000);

        // remove the client from the list of connected clients when the connection is closed or there is an error
        ws.on('close', () => {
            const index = clients.indexOf(ws);
            if (index !== -1) {
                clients[index].connected = false;
            }
            wsSpan.end();
        });
        ws.on('error', () => {
            const index = clients.indexOf(ws);
            if (index !== -1) {
                clients[index].connected = false;
            }
        });
    });
}


function follow_role() {

    const span = tracer.startSpan('follow_role', {
        kind: SpanKind.CLIENT,
    });

    // const _address = 'ws://localhost:8080'
    const _address = 'wss://10.0.0.111/ws'
    //const _address = 'ws://websocket-middleware.default.svc.cluster.local:8080'

    const ws = new WebSocket(_address, {
        rejectUnauthorized: false
    });
    const producer = new Kafka.Producer({
        'metadata.broker.list': bootstrapServers,
    }, {});

    producer.on('ready', () => {
        producer.setPollInterval(100);
    });

    producer.connect();

    ws.on('message', (message) => {
        messageCounter++;
        producer.produce(topics[0], null, Buffer.from(message), null);
    });

    ws.on('open', () => {
        console.log('Connected to WebSocket server');
        span.end();
    });

    ws.on('error', (err) => {
        console.log(`WebSocket error: ${err}`);
    });
}




if (direction === "LEAD") {
    lead_role();
} else {
    follow_role();
}
