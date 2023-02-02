const WebSocket = require('ws');

// const _address = 'ws://localhost:8080'
const _address = 'ws://websocket-middleware.default.svc.cluster.local:8080'

const ws = new WebSocket(_address);

ws.on('open', function open() {
    console.log('connected');
});

ws.on('message', function incoming(data) {
    try {
        const jsonData = JSON.parse(data);
        console.log(jsonData)
    } catch (e) {
        console.log('Error:', e);
    }
});

// Respond to liveliness checks
ws.on('ping', () => {
    console.log("Received ping from server, sending pong...");
    ws.pong();
});

ws.on('error', function (err) {
    console.log('Error:', err);
    // same code as in 'close' event listener
});

ws.on('close', function () {
    console.log('disconnected');
    // same code as in 'error' event listener
});

