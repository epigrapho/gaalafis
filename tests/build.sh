# build the docker images needed for the tests
docker build ../modules/lfs-server -t "gaalafis/lfs-server:0.0.1" -f ../modules/lfs-server/Dockerfile --target=runtime
docker build ../modules/auth -t "gaalafis/gitolite:0.0.1" -f ../modules/auth/Dockerfile 
docker build . -t "gaalafis:tester_client" -f ./runner/Dockerfile 
docker build ./architectures/nginx -t "gaalafis:nginx" -f ./architectures/nginx/Dockerfile
docker build ./architectures/nginx -t "gaalafis:nginx-no-bucket" -f ./architectures/nginx/no-bucket.Dockerfile

