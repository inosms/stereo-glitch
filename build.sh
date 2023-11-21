#!/bin/bash

echo "Building..."
wasm-pack build --target web

echo "Copying..."
mkdir -p site
cp pkg/*.js site/
cp pkg/*.wasm site/
cp static/* site/