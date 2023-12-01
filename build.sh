#!/bin/bash

echo "Building..."
# supply all command line arguments to the build command
wasm-pack build --target web $@                                                     