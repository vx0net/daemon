#!/bin/bash

# VX0 Network Certificate Generation Script
# This creates self-signed certificates for VX0 node authentication

set -e

CERTS_DIR="$HOME/vx0-network/certs"
HOSTNAME="${1:-$(hostname)}"

echo "=== VX0 Network Certificate Generation ==="
echo "Generating certificates for hostname: $HOSTNAME"
echo "Certificates will be stored in: $CERTS_DIR"
echo ""

# Create certificates directory
mkdir -p "$CERTS_DIR"
cd "$CERTS_DIR"

# Generate CA private key
if [ ! -f "ca.key" ]; then
    echo "🔐 Generating CA private key..."
    openssl genrsa -out ca.key 4096
fi

# Generate CA certificate
if [ ! -f "ca.crt" ]; then
    echo "📜 Generating CA certificate..."
    openssl req -new -x509 -days 3650 -key ca.key -out ca.crt -subj "/C=US/ST=VX0/L=Network/O=VX0Network/OU=CA/CN=VX0-CA"
fi

# Generate node private key
echo "🔑 Generating node private key for $HOSTNAME..."
openssl genrsa -out "${HOSTNAME}.key" 2048

# Generate node certificate signing request
echo "📝 Generating certificate signing request..."
openssl req -new -key "${HOSTNAME}.key" -out "${HOSTNAME}.csr" -subj "/C=US/ST=VX0/L=Network/O=VX0Network/OU=Node/CN=${HOSTNAME}"

# Generate node certificate signed by CA
echo "✍️  Generating node certificate..."
openssl x509 -req -in "${HOSTNAME}.csr" -CA ca.crt -CAkey ca.key -CAcreateserial -out "${HOSTNAME}.crt" -days 365

# Set appropriate permissions
chmod 600 *.key
chmod 644 *.crt

# Clean up CSR
rm "${HOSTNAME}.csr"

echo ""
echo "✅ Certificates generated successfully!"
echo ""
echo "Files created:"
echo "  CA Certificate: $CERTS_DIR/ca.crt"
echo "  CA Private Key: $CERTS_DIR/ca.key"
echo "  Node Certificate: $CERTS_DIR/${HOSTNAME}.crt"
echo "  Node Private Key: $CERTS_DIR/${HOSTNAME}.key"
echo ""
echo "⚠️  Important Security Notes:"
echo "1. Keep ca.key secure and backed up"
echo "2. Share ca.crt with all VX0 nodes for verification"
echo "3. Never share private keys (.key files)"
echo "4. Each node should have its own unique certificate"
echo ""
echo "To verify the certificate:"
echo "  openssl x509 -in ${HOSTNAME}.crt -text -noout"