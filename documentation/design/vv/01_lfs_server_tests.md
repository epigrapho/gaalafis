## LFS server testing

The LFS server is the most complex custom component of the system. It is also the one that is the most likely to be reused in other projects. So it is important to ensure that it is well tested.

### Unit testing the LFS server

The LFS server has typically:

- services that implements traits, injected from the top
- helpers directly used by the services or the controllers
- controllers that implements the routes
- server startup code

The unit tests focus on the implementation of the services and on the helpers. 

### Integration tests

The integration tests focus on the controllers and the server startup code. They are run with a real database, and a real minio server. The database and the bucket are created before each test with a random uuid name. 

Integration tests are grouped into files, each one representing a set of possible arguments of the server: 

- `proxy fs locks pg`
- `proxy fs`
- `proxy sbs locks pg`
- `proxy sbs`
- `signer sbs locks pg`
- `signer sbs`

On the other side, there are scenarios to be run. 

### Batch object nominal

The nominal batch object scenario is the following:

1) Try to download a non existent file
2) Get a link to upload the file, but with the wrong token
3) Get a link to upload the file
4) Parse json and get back the link and the token
5) Upload the file
6) Get a link to download the file
7) reparse the actions
8) Download the content of the file

(It is implemented 2 times to simplify the verification of links in the proxy and signer variant, as the links are very different, but both test the same scenario)

### Batch object exit directory attack

This scenario study an attack from someone trying to access an object using relative path "../../.../secret" to find other files outside of his own repo. 

1) Upload a first file so the repo directory exists
2) Create a file in a secret location (outside of the repo directory)
3) Try to batch download the secret file
4) It shall have been refused by now, but if there were a token, the download itself should fail too

### Locks nominal

The nominal locks scenario involves 2 users that create locks, list them, and delete them.

1) User 1 creates a lock, but with the download token
2) User 1 creates a lock, but with the upload token now
3) User 1 tries to create a lock on the same path, but it fails
4) User 2 tries to create a lock on the same path, but it fails
5) User 1 tries to create a lock on a different path, it works
6) User 1 list the locks with a download token
7) But it also works with upload token
8) And user2 see the same
9) We can filter locks by path
10) We can filter locks by id
11) User 2 create a lock
12) User 1 can list locks for verifications with a download token
13) User 2 can list locks for verifications with a download token, and get the reversed result
14) Create two more lock to test limits
15) We can limit to 3 locks
16) We can limit down to 1 lock, starting from the 4th
17) Limit of 0 means no locks
18) NaN limit should fail
19) When listing locks for verification, we can apply a limit that apply before the separation between ours and theirs, so a limit of 3 captures both 2 locks of ours and 1 lock of theirs
20) But for user2, with a limit of 2, we only get 2 locks of theirs
21) And then on next call, we get our lock and the next one of theirs
22) User1 unlock lock 1 with a download token, shall fail
23) User1 unlock lock 1 with an upload token, ok
24) So now lock 1 do not appear anymore
25) User1 try to unlock lock 1 again, but it fails, as not found
26) User1 try to unlock lock 3 of user 2, but it fails, as forbidden
26) User1 force unlock lock 3 of user 2, succeed
27) User1 unlock other locks, succeed
28) User1 can list locks and get nothing

This scenario is run against all architectures.
