import http from 'k6/http';
import { check } from 'k6';

const path = 'http://localhost:2626';

function health() {
    const res = http.get(`${path}/health`);
    check(res, {
        'get /health: is status 200': (r) => r.status === 200,
        'get /health: is body 200 OK': (r) => r.body.includes("200 OK")
    });
}

function queues() {
    const queue_path = `${path}/queues`;
    const post1 = http.post(queue_path, JSON.stringify({ id: "test:queue" }), { headers: { 'Content-Type': 'application/json' } })
    const post2 = http.post(queue_path, JSON.stringify({ id: "test:queue2" }), { headers: { 'Content-Type': 'application/json' } })
    check(post1, {
        'post /queues: is status 200': (r) => r.status === 200
    })
    check(post2, {
        'post /queues: is status 200': (r) => r.status === 200
    })
    const res = http.get(queue_path);

    check(res, {
        'get /queues: is status 200': (r) => r.status === 200,
        'get /queues: is body correct': (r) => JSON.parse(r.body)["queues"][0]["id"] === "test:queue" || JSON.parse(r.body)["queues"][0]["id"] === "test:queue2"
    });
    http.del(`${queue_path}/test:queue2`)
    const res2 = http.get(queue_path);
    check(res2, {
        'get /queues: is status 200': (r) => r.status === 200,
        'get /queues: is body correct': (r) => JSON.parse(r.body)["queues"][0]["id"] === "test:queue"
    });

    const res3 = http.get(`${queue_path}/test:queue`);
    check(res3, {
        'get /queues/test:queue: is status 200': (r) => r.status === 200,
        'get /queues/test:queue: is body correct': (r) => JSON.parse(r.body)["id"] === "test:queue"
    });
}

function events() {
    const events_path = `${path}/events`;
    const queue_path = `${path}/queues`;
    http.post(queue_path, JSON.stringify({ id: "test:events" }), { headers: { 'Content-Type': 'application/json' } });
    const post1 = http.post(events_path, JSON.stringify({
        "data": 'xyz==',
        "schedule_rules": [
            {
                "queue": "test:events",
                "schedule_at": Date.now() + 1000
            }
        ]
    }), { headers: { 'Content-Type': 'application/json' } })
    check(post1, {
        'post /events: is status 200': (r) => r.status === 200,
    })
}

export default function () {
    health();
    queues();
    events();
}
