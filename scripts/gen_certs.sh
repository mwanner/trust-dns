# !/bin/bash

set -e

OPENSSL=/usr/local/opt/openssl/bin/openssl

trust_dns_dir=$(dirname $0)/..

pushd $trust_dns_dir/tests

for i in ca.key ca.pem cert.key cert.csr cert.pem cert.p12 ; do
    [ -f $i ] && echo "$i exists" && exit 1;
done

echo 

cat <<-EOF > /tmp/ca.conf
[req]
prompt = no
req_extensions = req_ext
distinguished_name = dn

[dn]

C = US
ST = California
L = San Francisco
O = TRust-DNS
CN = root.example.com

[req_ext]
#basicConstraints = CA:TRUE
subjectAltName = @alt_names
 
[alt_names]
DNS.1 = root.example.com
EOF

# CA
echo "----> Generating CA <----"
${OPENSSL:?} genrsa -out ca.key 4096
${OPENSSL:?} req -x509 -new -nodes -key ca.key -days 365 -out ca.pem \
             -verify \
             -config /tmp/ca.conf


cat <<-EOF > /tmp/cert.conf
[req]
prompt = no
req_extensions = req_ext
distinguished_name = dn

[dn]

C = US
ST = California
L = San Francisco
O = TRust-DNS
CN = ns.example.com

[req_ext]

basicConstraints = CA:FALSE
keyUsage = nonRepudiation, digitalSignature, keyEncipherment
subjectAltName = @alt_names
 
[alt_names]
DNS.1 = ns.example.com
EOF

# Cert
echo "----> Generating CERT  <----"
${OPENSSL:?} genrsa -out cert.key 4096
${OPENSSL:?} req -new -nodes -key cert.key -out cert.csr \
             -verify \
             -config /tmp/cert.conf

echo "----> Signing Cert <----"
${OPENSSL:?} x509 -req -days 365 -in cert.csr -CA ca.pem -CAkey ca.key  -set_serial 0x8771f7bdee982fa6 -out cert.pem -extfile /tmp/cert.conf -extensions req_ext

echo "----> Createing PCKS12 <----"
${OPENSSL:?} pkcs12 -export -inkey cert.key -in cert.pem -out cert.p12 -passout pass:mypass -name ns.example.com -chain -CAfile ca.pem

popd