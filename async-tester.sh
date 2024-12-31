#!/bin/bash

curl -H"Content-Length: 10" -d"test" -X POST localhost:7878/api/add-note &
curl localhost:7878/api/hello
