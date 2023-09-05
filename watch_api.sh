#!/bin/bash
pgrep mora-server | xargs kill > /dev/null
cargo run mora-server &
echo "Started server, testing..."
sleep 5
k6 run k6/api.js | grep mora_check
pgrep mora-server | xargs kill > /dev/null
echo "done"
exit 0
