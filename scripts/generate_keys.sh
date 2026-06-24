#!/bin/bash
# Generate RS256 Keypair for JWT Signing

mkdir -p keys

# Generate 2048-bit RSA Private Key
openssl genrsa -out keys/private.pem 2048

# Extract Public Key
openssl rsa -in keys/private.pem -outform PEM -pubout -out keys/public.pem

echo "✅ RSA keypair generated successfully in the 'keys' directory:"
echo "   - keys/private.pem (Keep this secret!)"
echo "   - keys/public.pem  (Share this with other microservices)"
