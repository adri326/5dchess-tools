#! /bin/sh

# Fetch the latest 5d-chess-db
curl -sfo db.tar.gz https://storage.googleapis.com/db-chess-in-5d/db.tar.gz
tar -xf db.tar.gz

if [ ! -d "converted-db" ]; then
    mkdir converted-db
    mkdir converted-db/nonmate
    mkdir converted-db/checkmate
fi

if [ ! -d "/tmp/5dchess-tools" ]; then
    mkdir /tmp/5dchess-tools
    mkdir /tmp/5dchess-tools/nonmate
    mkdir /tmp/5dchess-tools/checkmate
fi

for game in 5d-chess-db/db/white_timeout/*.c5d 5d-chess-db/db/black_timeout/*.c5d; do
    if grep -E "standard|princess|defended_pawn|half_reflected" "$game" > /dev/null; then
        target=$(basename ${game%.c5d}.json)
        head -n 4 "$game" > "/tmp/5dchess-tools/nonmate/$(basename ${game})" # Remove last 4 lines of the file, as to get a position that isn't mate
        node 5dchess-notation convert alexbay json "/tmp/5dchess-tools/nonmate/$(basename ${game})" > "/tmp/5dchess-tools/nonmate/${target}" &&
            cp "/tmp/5dchess-tools/nonmate/${target}" "converted-db/nonmate/${target}" &&
            echo "Converted (nonmate): " $(basename ${game}) ||
            echo "Error in (nonmate): " $(basename ${game})
    fi
done

for game in 5d-chess-db/db/white/*.c5d 5d-chess-db/db/black/*.c5d; do
    if grep -E "standard|princess|defended_pawn|half_reflected" "$game" > /dev/null; then
        target=$(basename ${game%.c5d}.json)
        cp "$game" "/tmp/5dchess-tools/checkmate/$(basename ${game})"
        node 5dchess-notation convert alexbay json "/tmp/5dchess-tools/checkmate/$(basename ${game})" > "/tmp/5dchess-tools/checkmate/${target}" &&
            cp "/tmp/5dchess-tools/checkmate/${target}" "converted-db/checkmate/${target}" &&
            echo "Converted (checkmate): " $(basename ${game}) ||
            echo "Error in (checkmate): " $(basename ${game})
    fi
done
