
import WebSocket from 'ws';

const ws = new WebSocket('ws://localhost:4000/ws/events/category', {
    perMessageDeflate: false,

});
ws.on('message', function incoming(message) {
    var data = new Buffer.from(message)
    console.log(JSON.parse(data))
})