#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CERT_DIR="${ROOT_DIR}/infra/proxy/caddy/certs/dev"

CA_KEY="${CERT_DIR}/reqstly-dev-rootCA.key"
CA_CERT="${CERT_DIR}/reqstly-dev-rootCA.pem"
CA_SERIAL="${CERT_DIR}/reqstly-dev-rootCA.srl"
SERVER_KEY="${CERT_DIR}/reqstly-dev.key"
SERVER_CSR="${CERT_DIR}/reqstly-dev.csr"
SERVER_CERT="${CERT_DIR}/reqstly-dev.crt"
EXT_FILE="${CERT_DIR}/reqstly-dev.ext"

mkdir -p "${CERT_DIR}"

if [[ ! -f "${CA_KEY}" || ! -f "${CA_CERT}" ]]; then
  echo "Generating development root CA..."
  openssl genrsa -out "${CA_KEY}" 4096
  openssl req -x509 -new -nodes -key "${CA_KEY}" -sha256 -days 3650 \
    -out "${CA_CERT}" \
    -subj "/C=MV/ST=Maldives/L=Male/O=Reqstly Dev/CN=Reqstly Dev Root CA"
else
  echo "Using existing development root CA."
fi

cat >"${EXT_FILE}" <<'EOF'
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, keyEncipherment
extendedKeyUsage = serverAuth
subjectAltName = @alt_names

[alt_names]
DNS.1 = localhost
DNS.2 = api.localhost
DNS.3 = *.localhost
IP.1 = 127.0.0.1
EOF

echo "Generating server certificate for localhost and api.localhost..."
openssl genrsa -out "${SERVER_KEY}" 2048
openssl req -new -key "${SERVER_KEY}" -out "${SERVER_CSR}" -subj "/CN=localhost"
openssl x509 -req -in "${SERVER_CSR}" -CA "${CA_CERT}" -CAkey "${CA_KEY}" \
  -CAcreateserial -CAserial "${CA_SERIAL}" -out "${SERVER_CERT}" \
  -days 825 -sha256 -extfile "${EXT_FILE}"

rm -f "${SERVER_CSR}" "${EXT_FILE}"

cat <<EOF

Dev certificates generated:
- Root CA: ${CA_CERT}
- Server certificate: ${SERVER_CERT}
- Server key: ${SERVER_KEY}

To trust this CA on macOS (one-time):
sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain "${CA_CERT}"

EOF
