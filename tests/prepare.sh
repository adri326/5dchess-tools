#! /bin/sh

# Fetch the latest 5d-chess-db
curl -sfo db.tar.gz https://storage.googleapis.com/db-chess-in-5d/db.tar.gz
tar -xf db.tar.gz

if [ ! -d "converted-db" ]; then
    mkdir converted-db
fi

if [ ! -d "/tmp/5dchess-tools" ]; then
    mkdir /tmp/5dchess-tools
fi

for game in 5d-chess-db/db/white_timeout/*.c5d 5d-chess-db/db/black_timeout/*.c5d; do
    if grep -E "standard|princess|defended_pawn|half_reflected" "$game" > /dev/null; then
        target=$(basename ${game%.c5d}.json)
        head -n 4 "$game" > "/tmp/5dchess-tools/$(basename ${game})" # Remove last 4 lines of the file, as to get a position that isn't mate
        node 5dchess-notation convert alexbay json "/tmp/5dchess-tools/$(basename ${game})" > "/tmp/5dchess-tools/${target}" &&
            cp "/tmp/5dchess-tools/${target}" "converted-db/${target}" &&
            echo "Converted: " $(basename ${game}) ||
            echo "Error in: " $(basename ${game})
    fi
done
