#!/usr/bin/env bash
set -x

# Set up authentication:
API_KEY_CONTENT=$(cat "./APIKey.pem")
PRIVATE_KEY_PATH="../test-prv-key.pem"

# Set up the request:
API_METHOD="GET"
API_CALL="api/v3/account"

# Sign the request:
timestamp=$(date +%s000)
api_params_with_timestamp="timestamp=$timestamp"
signature=$(echo -n "$api_params_with_timestamp" \
            | openssl dgst -sha256 -sign "$PRIVATE_KEY_PATH" \
            | openssl enc -base64 -A)

# Send the request:
curl -H "X-MBX-APIKEY: $API_KEY_CONTENT" -X "$API_METHOD" \
    "https://testnet.binance.vision/$API_CALL?$api_params_with_timestamp" \
    --data-urlencode "signature=$signature"