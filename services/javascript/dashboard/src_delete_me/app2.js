const Plotly = require('plotly.js-dist');
const WebSocket = require('ws');

// data arrays for the line chart
let x = [];
let y = [];
let topicColors = {}; // store a color for each topic

// initialize the line chart
const layout = {
    title: 'Kafka Topic Polling',
    xaxis: {
        title: 'Index'
    },
    yaxis: {
        title: 'Value'
    }
};

const data = [{
    type: 'scatter',
    mode: 'lines',
    x,
    y,
    line: {
        color: 'black'
    }
}];

Plotly.newPlot('chart', data, layout);

// connect to the websocket
const ws = new WebSocket('ws://localhost:8080');

ws.on('message', function incoming(data) {
    const event = JSON.parse(data);

    if (!topicColors[event.topic]) {
        topicColors[event.topic] = Plotly.d3.schemeCategory10[Object.keys(topicColors).length % 10];
    }
    data[0].line.color = topicColors[event.topic];
    x.push(event.index);
    y.push(event.value);
    Plotly.update('chart', { x, y, data });

    // if there are more than 100 x values, remove the oldest one
    if (x.length > 100) {
        x.shift();
        y.shift();
    }
});
