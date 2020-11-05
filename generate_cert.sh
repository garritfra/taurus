# Generates a test-certificate

# When prompted for multiple lines of information, leave everything blank instead of "common name"
# This should be your domain name. E.g. "localhost" if you are testing on your local machine

openssl genrsa -des3 -out server.key 1024
openssl req -new -key server.key -out server.csr
openssl x509 -req -days 1024 -in server.csr -signkey server.key -out server.crt
openssl pkcs12 -export -out identity.pfx -inkey server.key -in server.crt