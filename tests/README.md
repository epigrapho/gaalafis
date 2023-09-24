# Run the test

To run the tests,

- Make sure you have docker installed. 
- Run `./make_secrets.sh`. It will
    - Generate a new key pair `id_rsa` and `id_rsa.pub` in the `runner/ssh` folder, so we can test ssh connexions to the git server.
    - Generate a self-signed certificate `nginx-selfsigned.crt` and `nginx-selfsigned.key` in the `architectures/nginx/ssl` folder, so we can test https connexions.
    - Other secrets (for the jwt, the minio test instance, ...) are simple default ones that are used in the tests, and are not meant to be used in production. They are not protected and are available in the `architecture/secrets` folder. The jwt secret for instance is assumed to be `secret` for all tests that have already signed jwt tokens in their fixtures.
- Make sure you build the images. From the `/tests` folder, you can run `./build.sh`
- Run test 23 for instance using `./start.sh 23`

To run all tests, you can use `./build_and_test.sh`. It will rebuild the images and run all tests in order.
