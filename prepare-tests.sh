#! /bin/bash

# Fetch the latest 5d-chess-db
echo "Fetching database..."

curl -fo 5d-chess-db-converted.tar.gz https://www.shadamethyst.xyz/mirror/5d-chess-db-converted.tar.gz

if [ -d converted-db ]; then
    echo "Removing old database"
    rm -r converted-db
fi

echo "Uncompressing database..."

tar -xf 5d-chess-db-converted.tar.gz

echo "Done!"
