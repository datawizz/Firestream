const WebSocket = require('ws');
const ws = new WebSocket('ws://localhost:8080');

ws.on('open', function open() {
    console.log('connected');
});

ws.on('message', function incoming(data) {
    try {
        const jsonData = JSON.parse(data);
        // const jsonPayload = JSON.parse(jsonData.value)
        // if (jsonData.topic != 'metronome') {
        //     console.log(jsonData);
        // }
        console.log(jsonData)
    } catch (e) {
        console.log('Error:', e);
    }
});
