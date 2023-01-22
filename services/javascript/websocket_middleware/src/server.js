

// BOOTSTRAP_SERVERS = "kafka.default.svc.cluster.local:9092"
// kafka-zookeeper.default.svc.cluster.local:2181



const { Kafka } = require('kafkajs')
const WebSocket = require('ws');

const kafka = new Kafka({
    clientId: 'websocket_middleware',
    brokers: ['kafka.default.svc.cluster.local:9092']
})
const consumer = kafka.consumer({ groupId: 'my-group3' })

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
            if (topicArrays[topic].length > 100) {
                topicArrays[topic].shift();
            }
        },
    })
}



consumeMessages();

// websocket server setup
const wss = new WebSocket.Server({ port: 8080 });
const clients = [];
// wss.on('connection', (ws) => {
//     // add the new client to the list of connected clients
//     clients.push(ws);

//     // send the current message queue to the new websocket client
//     topics.forEach(topic => {
//         ws.send(JSON.stringify(topicArrays[topic]));
//     });

//     // send new messages to all connected websocket clients as they come in
//     setInterval(() => {
//         topics.forEach(topic => {
//             while (topicArrays[topic].length > 0) {
//                 clients.forEach(client => {
//                     client.send(JSON.stringify(topicArrays[topic].shift()));
//                 });
//             }
//         });
//     }, 100);

//     // remove the client from the list of connected clients when the connection is closed
//     ws.on('close', () => {
//         const index = clients.indexOf(ws);
//         if (index !== -1) {
//             clients.splice(index, 1);
//         }
//     });
// });

wss.on('connection', (ws) => {
    // add the new client to the list of connected clients
    clients.push({ ws, connected: true });
    // send the current message queue to the new websocket client
    topics.forEach(topic => {
        ws.send(JSON.stringify(topicArrays[topic]));
    });
    setInterval(() => {
        topics.forEach(topic => {
            while (topicArrays[topic].length > 0) {
                clients.forEach(client => {
                    if (client.connected) client.ws.send(JSON.stringify(topicArrays[topic].shift()));
                });
            }
        });
    }, 100);
    // remove the client from the list of connected clients when the connection is closed
    ws.on('close', () => {
        const index = clients.indexOf(ws);
        if (index !== -1) {
            clients[index].connected = false;
        }
    });
});
