#! /bin/bash

cargo build --release --example monte_carlo

ctrl_c() {
    killall monte_carlo
    exit
}

if [ ! -d "/tmp/db" ]; then
    mkdir /tmp/db
fi
if [ ! -d "./db" ]; then
    mkdir ./db
fi

if [ ! -d "/tmp/db/standard" ]; then
    mkdir /tmp/db/standard
fi

if [ ! -d "/tmp/db/standard/white_timeout" ]; then
    mkdir /tmp/db/standard/white_timeout
fi
if [ ! -d "/tmp/db/standard/white" ]; then
    mkdir /tmp/db/standard/white
fi
if [ ! -d "/tmp/db/standard/black_timeout" ]; then
    mkdir /tmp/db/standard/black_timeout
fi
if [ ! -d "/tmp/db/standard/black" ]; then
    mkdir /tmp/db/standard/black
fi
if [ ! -d "/tmp/db/standard/stalemate" ]; then
    mkdir /tmp/db/standard/stalemate
fi
if [ ! -d "/tmp/db/standard/stalemate_timeout" ]; then
    mkdir /tmp/db/standard/stalemate_timeout
fi
if [ ! -d "/tmp/db/standard/none" ]; then
    mkdir /tmp/db/standard/none
fi

let N=$(lscpu | grep -E "^CPU\\(s\\):" | grep -Po "\\d+")-1
echo "Starting ${N} threads!"
trap ctrl_c INT

for thread in $(seq 1 $N); do
    ./target/release/examples/monte_carlo &
done

sleep 5

while true; do
    echo "Pushing to cloud"

    for game in /tmp/db/*/*/*.5dpgn; do
        (echo "${game}"; node 5dchess-notation convert shad shad "${game}") > /tmp/db/curr.5dpgn &&
        node examples/monte_carlo/hash.js &&
        rm "${game}"
    done

    # gsutil -m rsync -r ./db/ gs://db-chess-in-5d/db/

    sleep 120
done

killall monte_carlo
