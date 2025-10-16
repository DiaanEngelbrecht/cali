#!/bin/bash

# Script to generate TypeScript clients from proto files
# This mirrors the backend's controller generation workflow

set -e

PROTO_DIR="../interface/grpc/services"
OUT_DIR="src/lib/generated"

# Create output directory if it doesn't exist
mkdir -p "$OUT_DIR"

# Find all proto files
PROTO_FILES=$(find "$PROTO_DIR" -name "*.proto")

if [ -z "$PROTO_FILES" ]; then
    echo "No proto files found in $PROTO_DIR"
    exit 1
fi

echo "Generating TypeScript clients from proto files..."

# Generate TypeScript code using buf's protoc plugins
protoc \
    --plugin=./node_modules/.bin/protoc-gen-es \
    --plugin=./node_modules/.bin/protoc-gen-connect-es \
    --es_out="$OUT_DIR" \
    --es_opt=target=ts \
    --connect-es_out="$OUT_DIR" \
    --connect-es_opt=target=ts \
    --proto_path="$PROTO_DIR" \
    $PROTO_FILES

echo "✓ Generated TypeScript clients in $OUT_DIR"
