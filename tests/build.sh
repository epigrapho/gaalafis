# build the docker images needed for the tests
docker build ../modules/lfs-server -t "gaalafis/lfs-server:0.0.1-proxy-fs" -f ../modules/lfs-server/Dockerfile --target=runtime-proxy-fs
docker build ../modules/lfs-server -t "gaalafis/lfs-server:0.0.1-proxy-fs-locks-pg" -f ../modules/lfs-server/Dockerfile --target=runtime-proxy-fs-locks-pg
docker build ../modules/lfs-server -t "gaalafis/lfs-server:0.0.1-proxy-sbs" -f ../modules/lfs-server/Dockerfile --target=runtime-proxy-sbs
docker build ../modules/lfs-server -t "gaalafis/lfs-server:0.0.1-proxy-sbs-locks-pg" -f ../modules/lfs-server/Dockerfile --target=runtime-proxy-sbs-locks-pg
docker build ../modules/lfs-server -t "gaalafis/lfs-server:0.0.1-signer-sbs" -f ../modules/lfs-server/Dockerfile --target=runtime-signer-sbs
docker build ../modules/lfs-server -t "gaalafis/lfs-server:0.0.1-signer-sbs-locks-pg" -f ../modules/lfs-server/Dockerfile --target=runtime-signer-sbs-locks-pg

docker build ../modules/auth -t "gaalafis/gitolite:0.0.1" -f ../modules/auth/Dockerfile 

docker build . -t "gaalafis:tester_client" -f ./runner/Dockerfile 

docker build ./architectures/nginx -t "gaalafis:nginx" -f ./architectures/nginx/Dockerfile
docker build ./architectures/nginx -t "gaalafis:nginx-no-bucket" -f ./architectures/nginx/no-bucket.Dockerfile

