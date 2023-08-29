const WebSocket = require('ws');
const express = require('express');
const http = require('http');
const Kafka = require('node-rdkafka');

// Health Checks Server
const app = express();
const server = http.createServer(app);
app.get('/health', (req, res) => {
    res.send('OK');
});
server.listen(8000);

// websocket server setup
const wss = new WebSocket.Server({ port: 8080 });
const clients = [];

let messageCounter = 0;
setInterval(() => {
    console.log(`Kafka messages received in the last 5 seconds: ${messageCounter}`);
    messageCounter = 0;
}, 5000);

const debug = false
const bootstrapServers = 'kafka.default.svc.cluster.local:9092'

const connectToKafka = (consumer) => {
    consumer.connect();

    consumer
        .on('ready', () => {
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

const topics = ['A_RUSTY_TOPIC'];
const topicArrays = {};
topics.forEach(topic => {
    topicArrays[topic] = [];
});

connectToKafka(consumer);

wss.on('connection', (ws) => {
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
    });
    ws.on('error', () => {
        const index = clients.indexOf(ws);
        if (index !== -1) {
            clients[index].connected = false;
        }
    });
});
