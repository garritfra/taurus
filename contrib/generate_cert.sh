# Generates a test-certificate

# When prompted for multiple lines of information, leave everything blank instead of "common name"
# This should be your domain name. E.g. "localhost" if you are testing on your local machine

openssl genrsa -des3 -out server.key 4096
openssl req -new -key server.key -out server.csr
openssl x509 -req -days 4096 -in server.csr -signkey server.key -out server.crt
openssl pkcs12 -export -out identity.pfx -inkey server.key -in server.crt

# Clean up
rm server.key server.csr server.crt
