#!/bin/bash
pgrep mora-server | xargs kill > /dev/null
cargo run mora-server &
echo "Started server, testing..."
sleep 6
k6 run k6/api.js -u 1 -i 1 --batch 1 | grep mora_check
pgrep mora-server | xargs kill > /dev/null
echo "done"
exit 0
