import http from 'k6/http';
import { check } from 'k6';

const path = 'http://localhost:2626';

function health() {
    const res = http.get(`${path}/health`);
    check(res, {
        'mora_check: get /health: is status 200': (r) => r.status === 200,
        'mora_check: get /health: is body 200 OK': (r) => r.body.includes("200 OK")
    });
}

function queues() {
    const queue_path = `${path}/queues`;
    const post1 = http.post(queue_path, JSON.stringify({ id: "test:queue" }), { headers: { 'Content-Type': 'application/json' } })
    const post2 = http.post(queue_path, JSON.stringify({ id: "test:queue2" }), { headers: { 'Content-Type': 'application/json' } })
    check(post1, {
        'mora_check: post /queues: test:queue is status 200': (r) => r.status === 200
    })
    check(post2, {
        'mora_check: post /queues: test:queues2 is status 200': (r) => r.status === 200
    })
    const res = http.get(queue_path);

    check(res, {
        'mora_check: get /queues: is status 200': (r) => r.status === 200,
        'mora_check: get /queues: is body correct': (r) => JSON.parse(r.body)["queues"][0]["id"] === "test:queue" || JSON.parse(r.body)["queues"][0]["id"] === "test:queue2"
    });
    http.del(`${queue_path}/test:queue2`)
    const res2 = http.get(queue_path);
    check(res2, {
        'mora_check: get /queues: is status 200': (r) => r.status === 200,
        'mora_check: get /queues: is body correct': (r) => JSON.parse(r.body)["queues"][0]["id"] === "test:queue"
    });

    const res3 = http.get(`${queue_path}/test:queue`);
    check(res3, {
        'mora_check: get /queues/test:queue: is status 200': (r) => r.status === 200,
        'mora_check: get /queues/test:queue: is body correct': (r) => JSON.parse(r.body)["id"] === "test:queue"
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
                "schedule_for": Date.now() + 1000
            }
        ]
    }), { headers: { 'Content-Type': 'application/json' } })
    check(post1, {
        'mora_check: post /events: is status 200': (r) => r.status === 200,
    })
}

function channels() {
    const channels_path = `${path}/channels`;
    const events_path = `${path}/events`;
    const queue_path = `${path}/queues`;
    http.post(queue_path, JSON.stringify({
        id: "test:channels"
    }), { headers: { 'Content-Type': 'application/json' } });
    const post1 = http.post(channels_path, JSON.stringify({
        buffer_options: {
            time: 1000,
            size: 10,
        }, queues: ["test:channels"]
    }), { headers: { 'Content-Type': 'application/json' } });
    check(post1, {
        'mora_check: post /channels: is status 200': (r) => r.status === 200,
    })
    const schedule_date = Date.now() + 1000;
    http.post(events_path, JSON.stringify({
        "data": 'xyz==',
        "schedule_rules": [
            {
                "queue": "test:channels",
                "schedule_for": schedule_date
            }
        ]
    }), { headers: { 'Content-Type': 'application/json' } })
    const channel_id = JSON.parse(post1.body).channel_id;
    const get1 = http.get(`${channels_path}`);
    check(get1, {
        'mora_check: get /channels: is status 200': (r) => r.status === 200,
        'mora_check: get /channels: is body correct': (r) => r.body === JSON.stringify({
            channels: [channel_id]
        })
    });
    const get2 = http.get(`${channels_path}/${channel_id}`);
    check(get2, {
        'mora_check: get /channels/{channel_id}: is status 200': (r) => r.status === 200,
        'mora_check: get /channels/{channel_id}: is body correct': (r) => r.body === JSON.stringify({
            channel_id: channel_id,
            queues: ["test:channels"],
            buffer_options: {
                time: 1000,
                size: 10,
            },
        })
    });

    const get3 = http.get(`${channels_path}/${channel_id}/events`);
    check(get3, {
        'mora_check: get /channels/{channel_id}/events: is status 200': (r) => r.status === 200,
        'mora_check: get /channels/{channel_id}/events: is body correct': (r) => r.body === JSON.stringify({
            events: [
                {
                    data: "xyz=="
                }
            ]
        })
    });
    const get4 = http.get(`${channels_path}/${channel_id}/events`);
    check(get4, {
        'mora_check: get /channels/{channel_id}/events: is status 200': (r) => r.status === 200,
        'mora_check: get /channels/{channel_id}/events: is body empty after dequeued': (r) => r.body === JSON.stringify({
            events: []
        })
    });

    const del1 = http.del(`${channels_path}/${channel_id}`);
    check(del1, {
        'mora_check: delete /channels/{channel_id}: is status 200': (r) => r.status === 200,
        'mora_check: delete /channels/{channel_id}: is body correct': (r) => r.body === ""
    });
    const get5 = http.get(`${channels_path}/${channel_id}`);
    check(get5, {
        'mora_check: delete /channels/{channel_id}: is status 200': (r) => r.status === 404,
        'mora_check: delete /channels/{channel_id}: is body correct': (r) => r.body === `${channel_id} channel does not exist`
    });
}

export default function () {
    health();
    queues();
    events();
    channels();
}
