# build the docker images needed for the tests
docker build ../modules/lfs-server -t "gaalafis:lfs-server" -f ../modules/lfs-server/Dockerfile
docker build ../modules/auth -t "gaalafis:gitolite" -f ../modules/auth/Dockerfile 
docker build . -t "gaalafis:tester_client" -f ./runner/Dockerfile 
docker build ./architectures/nginx -t "gaalafis:nginx" -f ./architectures/nginx/Dockerfile
