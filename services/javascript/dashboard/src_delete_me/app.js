const Plotly = require('plotly.js-dist');
const WebSocket = require('ws');

// data arrays for the line chart
let x = [];
let y = [];

// websocket client setup
const ws = new WebSocket('ws://localhost:8080');
ws.on('message', (message) => {
    let data = JSON.parse(message);
    x.push(data.index);
    y.push(data.value);
    // To display the event_time with each index string 
    let annotations = [{
        x: data.index,
        y: data.value,
        text: data.event_time,
        xanchor: 'center',
        yanchor: 'bottom',
        showarrow: true
    }]
    // To color the lines according to the topic
    let color = '';
    switch (data.topic) {
        case 'topic1':
            color = 'red';
            break;
        case 'topic2':
            color = 'blue';
            break;
        // Add cases for other topics
    }
    console.log(x, y)
    Plotly.update('chart', { x, y }, { annotations: annotations }, { line: { color: color } });
});

// initialize the line chart
const layout = {
    title: 'Kafka Topic Polling',
    xaxis: {
        title: 'Time'
    },
    yaxis: {
        title: 'Value'
    }
};