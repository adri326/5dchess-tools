#! /bin/bash

# Fetch the latest 5d-chess-db
echo "Fetching database..."

curl -fo 5d-chess-db-converted.tar.gz https://www.shadamethyst.xyz/mirror/5d-chess-db-converted.tar.gz

echo "Uncompressing database..."

tar -xf 5d-chess-db-converted.tar.gz

echo "Done!"
