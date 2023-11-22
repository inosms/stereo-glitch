#!/bin/bash

echo "Building..."
# supply all command line arguments to the build command
wasm-pack build --target web $@                                                     

echo "Copying..."
mkdir -p site
cp pkg/*.js site/
cp pkg/*.wasm site/
cp static/* site/