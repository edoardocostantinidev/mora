import http from 'k6/http';
import { sleep, check } from 'k6';
export default function () {
    const res = http.get('http://localhost:2626/health');
    check(res, {
        'is status 200': (r) => r.status === 200,
        'is body 200 OK': (r) => r.body.includes("200 OK")
    });
    sleep(1);
}
