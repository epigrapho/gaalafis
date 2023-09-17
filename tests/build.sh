# build the docker images needed for the tests
docker build ../modules/auth -t "gaalafis:gitolite" -f ../modules/auth/Dockerfile 
docker build . -t "gaalafis:tester_client" -f ./runner/Dockerfile 
