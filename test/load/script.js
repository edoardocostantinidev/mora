import http from "k6/http";

export let options = {
    vus: 100,
    stages: [
        { duration: "1m", target: 25 },
        { duration: "2m", target: 50 },
        { duration: "3m", target: 100 },
        { duration: "1m", target: 0 },
    ]
};
export default function () {
    var date = new Date();
    var time_milliseconds = date.getTime();
    var time_milliseconds_postponed = time_milliseconds + (1000 * 60 * 60 * 24);

    var postData = `[
    {
        "createdAt": ${time_milliseconds},
        "fireAt": ${time_milliseconds_postponed},
        "category": "category",
        "data": {
            "field": "value"
        }
    }]`;
    let port = __ENV.PORT || 8000;
    let response = http.post(`http://localhost:${port}/events`, postData);
};