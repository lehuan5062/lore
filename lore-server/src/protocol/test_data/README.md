# Test Certificates

This directory contains certificates used to spin up a Lore server for use in tests. As such, this
directory is ignored by the pre-commit hook that scans for private keys.

## Files

| File                    | Purpose                                                               |
|-------------------------|-----------------------------------------------------------------------|
| `test_ca.pem`           | CA certificate used by the server to verify client certificates       |
| `test_cert.pem`         | Server certificate for TLS identity                                   |
| `test_key.pem`          | Private key for `test_cert.pem`                                       |
| `test_client_cert.pem`  | Client certificate signed by trusted CA (for mTLS acceptance testing) |
| `test_client_key.pem`   | Private key for `test_client_cert.pem`                                |
| `untrusted_ca.pem`      | A different CA not trusted by the server (for mTLS rejection testing) |
| `untrusted_cert.pem`    | Client certificate signed by `untrusted_ca.pem` (should be rejected)  |
| `untrusted_key.pem`     | Private key for `untrusted_cert.pem`                                  |

## Regenerating Certificates

Certificates are generated using the shared `make-certs.sh` script. Run from the repository root:

```bash
# Generate certs to a temp directory (enter any passphrase when prompted, e.g., "test")
scripts/server/make-certs.sh /tmp/urc-certs

# Copy and rename to test_data
cd urc-server/src/protocol/test_data
cp /tmp/urc-certs/ca.crt test_ca.pem
cp /tmp/urc-certs/server.crt test_cert.pem
cp /tmp/urc-certs/server.key test_key.pem
cp /tmp/urc-certs/client.crt test_client_cert.pem
cp /tmp/urc-certs/client.key test_client_key.pem
cp /tmp/urc-certs/ca-bad.crt untrusted_ca.pem
cp /tmp/urc-certs/client-bad.crt untrusted_cert.pem
cp /tmp/urc-certs/client-bad.key untrusted_key.pem

# Clean up
rm -rf /tmp/urc-certs
```
