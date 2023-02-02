// A Middleware to receive messages from Kafka and send to websocket clients

const { Kafka } = require('kafkajs')
const WebSocket = require('ws');

const debug = false

const kafka = new Kafka({
    clientId: 'websocket_middleware',
    brokers: ['kafka.default.svc.cluster.local:9092']
})
const consumer = kafka.consumer({ groupId: 'WebSocketMiddleware' })

// const topics = ['metronome', 'spark_structured_streaming_stateful']
// const topics = ['pyspark_wiener_process_stateless']
const topics = ['metronome']

// store the last 100 records for each topic in an array
const topicArrays = {};
topics.forEach(topic => {
    topicArrays[topic] = [];
});

// async function to consume messages from kafka and add them to topic arrays
const consumeMessages = async () => {
    await consumer.connect()
    topics.forEach(async topic => {
        await consumer.subscribe({ topic: topic, fromBeginning: false })
    });
    await consumer.run({
        eachMessage: async ({ topic, partition, message }) => {
            let key = message.key;
            let value = message.value;
            if (key != null) {
                key = key.toString();
            }
            if (value != null) {
                value = JSON.parse(value);
            }
            decoded_message = {
                topic,
                partition,
                key,
                value
            }
            topicArrays[topic].push(decoded_message);
            // Verbose logging
            if (debug) {
                console.log(topic);
                console.log(topicArrays[topic].length);
            }
            if (topicArrays[topic].length > 100) {
                topicArrays[topic].pop();
            }
        },
    })
}



consumeMessages();

// websocket server setup
const wss = new WebSocket.Server({ port: 8080 });
const clients = [];


wss.on('connection', (ws) => {
    // add the new client to the list of connected clients
    clients.push({ ws, connected: true, lastPong: new Date() });

    // Send data to all connected clients from queue
    setInterval(() => {
        topics.forEach(topic => {
            while (topicArrays[topic].length > 0) {
                // Remove the first element of the queue
                const _message = topicArrays[topic].shift();
                // Send the first element to all connected clients
                clients.forEach(client => {
                    // console.log(JSON.stringify(topicArrays[topic][0]).toString());
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
