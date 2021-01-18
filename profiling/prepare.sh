#! /bin/sh

# Fetch the latest 5d-chess-db
curl -sfo db.tar.gz https://storage.googleapis.com/db-chess-in-5d/db.tar.gz
tar -xf db.tar.gz

if [ ! -d "converted-db" ]; then
    mkdir converted-db
fi

for game in 5d-chess-db/db/white_timeout/*.c5d 5d-chess-db/db/black_timeout/*.c5d; do
    if grep "standard" "$game" > /dev/null; then
        node 5dchess-notation convert alexbay json "$game" > "converted-db/$(basename ${game%.c5d}.json)" && echo $(basename ${game%.c5d})
    fi
done
