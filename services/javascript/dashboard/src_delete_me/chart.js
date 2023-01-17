const Plotly = require('plotly.js-dist');

// data arrays for the line chart
let x = [];
let y = [];

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
const data = [{
    type: 'scatter',
    mode: 'lines',
    x,
    y
}];

Plotly.newPlot('chart', data, layout);

// poll the kafka topic and update the chart
consumer.on('message', (message) => {
    x.push(new Date());
    y.push(message.value);
    Plotly.update('chart', { x, y });
});