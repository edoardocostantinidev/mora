import { randomInt } from 'crypto';
import http from 'http'
async function main() {
    let jobs = process.argv[2] || 1;
    let sleep_ms = process.argv[3] || 100;
    let fns = [];
    for (let j = 0; j < jobs; j++) {
        fns.push(generate(sleep_ms));
    }
    Promise.all(fns)
        .then(results => console.log(results));

}
async function generate(sleep_ms) {
    await sleep(randomInt(120));
    for (let index = 0; index < 10000000; index++) {

        var date = new Date();
        var time_milliseconds = date.getTime();
        var time_milliseconds_postponed = time_milliseconds + (1000 * 1);

        var options = {
            'method': 'POST',
            'hostname': 'localhost',
            'port': 4000,
            'path': '/events',
            'headers': {
                'Content-Type': 'application/json'
            },
            'maxRedirects': 20
        };

        var req = http.request(options, function (res) {
            var chunks = [];

            res.on("data", function (chunk) {
                chunks.push(chunk);
            });

            res.on("end", function (chunk) {
                var body = Buffer.concat(chunks);
                console.log(body.toString());
            });

            res.on("error", function (error) {
                console.error(error);
            });
        });

        var postData = `[
    {
        "createdAt": ${time_milliseconds},
        "fireAt": ${time_milliseconds_postponed},
        "category": "category",
        "data": {
            "field": "value"
        }
    }
]`;
        console.log(index);
        req.write(postData);

        req.end();
        await sleep(sleep_ms);
    }
}
function sleep(ms) {
    return new Promise((resolve) => {
        setTimeout(resolve, ms);
    });
}

main().then().catch()
