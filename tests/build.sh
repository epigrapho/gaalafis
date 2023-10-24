# build the docker images needed for the tests
docker build ../modules/lfs-server -t "gaalafis/lfs-server:0.0.1-proxy-sbs" -f ../modules/lfs-server/Dockerfile --target=runtime_sbs_custom_signing
docker build ../modules/lfs-server -t "gaalafis/lfs-server:0.0.1-proxy-sbs-locks-pg" -f ../modules/lfs-server/Dockerfile --target=runtime_sbs_custom_signing_locks
docker build ../modules/lfs-server -t "gaalafis/lfs-server:0.0.1-signer-sbs-locks" -f ../modules/lfs-server/Dockerfile --target=runtime_main
docker build ../modules/auth -t "gaalafis/gitolite:0.0.1" -f ../modules/auth/Dockerfile 
docker build . -t "gaalafis:tester_client" -f ./runner/Dockerfile 
docker build ./architectures/nginx -t "gaalafis:nginx" -f ./architectures/nginx/Dockerfile
