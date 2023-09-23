# Run the test

To run the tests,

- Make sure you have docker installed. 
- Generate a new key paire `id_rsa` and `id_rsa.pub` in the `runner/ssh` folder.
- Make sure you build the images. From the `/tests` folder, you can run `./build.sh`
- Run test 23 for instance using `./start.sh 23`

To make test easier, some fixture secrets are provided in the `architecture/secrets`. These are assumed in the tests. 
