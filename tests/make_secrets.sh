# generate ssh key of the client tester
ssh-keygen -t rsa -f runner/ssh/id_rsa
ssh-keygen -t rsa -f runner/ssh/id_rsa_2

# generate ssl certificate for nginx reverse proxy
mkdir -p ./architectures/nginx/ssl
openssl req -new -newkey rsa:2048 -days 365 -nodes -x509 -keyout ./architectures/nginx/ssl/nginx-selfsigned.key -out ./architectures/nginx/ssl/nginx-selfsigned.crt -subj "/CN=proxy"
