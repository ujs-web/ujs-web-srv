#!/bin/bash

echo "Testing JSON-RPC API..."

echo ""
echo "1. Testing add method:"
curl -s -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"add","params":{"a":5,"b":3},"id":1}' \
  http://localhost:3001/rpc | jq .

echo ""
echo "2. Testing multiply method:"
curl -s -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"multiply","params":{"a":4,"b":7},"id":2}' \
  http://localhost:3001/rpc | jq .

echo ""
echo "3. Testing greet method:"
curl -s -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"greet","params":{"name":"World"},"id":3}' \
  http://localhost:3001/rpc | jq .

echo ""
echo "4. Testing method not found:"
curl -s -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"non_existent","params":{},"id":4}' \
  http://localhost:3001/rpc | jq .

echo ""
echo "5. Testing invalid JSON:"
curl -s -X POST \
  -H "Content-Type: application/json" \
  -d '{invalid json}' \
  http://localhost:3001/rpc | jq .

echo ""
echo "All tests completed!"
