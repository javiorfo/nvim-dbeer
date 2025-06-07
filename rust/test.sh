#!/usr/bin/env bash

cargo run -- -engine postgres -conn-str "host=localhost user=admin password=admin dbname=db_dummy" -queries "select * from dummies" -dest-folder /tmp -border-style 2 -action 1 -header-style-link Boolean -dbeer-log-file /tmp/dbeer.log -dbname basename -log-debug true
