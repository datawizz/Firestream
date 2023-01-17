const http = require('http');
const fs = require('fs');
const port = 8081;

const requestHandler = (request, response) => {
    fs.readFile('/workspace/services/javascript/dashboard/src/index.html', (err, data) => {
        if (err) throw err;
        response.end(data);
    });
}

const server = http.createServer(requestHandler);

server.listen(port, (err) => {
    if (err) {
        return console.log(`Error: ${err}`);
    }
    console.log(`Server is listening on ${port}`);
});
