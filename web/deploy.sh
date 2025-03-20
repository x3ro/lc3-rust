#!/bin/bash
set -ex

TIMESTAMP=$(date +"%s")

ssh x3ro.de <<ENDSSH
    cd ~/www/lc3_x3ro_de/ && \
        mkdir -p releases/$TIMESTAMP
ENDSSH

rm -rf ./dist-prod &&
    NODE_ENV=production npm run build &&
    rsync -avz dist-prod/* x3ro.de:~/www/lc3_x3ro_de/releases/$TIMESTAMP

ssh x3ro.de <<ENDSSH
    cd ~/www/lc3_x3ro_de/ && \
        rm -f www && \
        ln -s releases/$TIMESTAMP www
ENDSSH
