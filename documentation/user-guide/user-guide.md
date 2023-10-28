# Use GAALAFIS as your git backend server

## Introduction 

GAALAFIS is a reference architecture to serve git and git-lfs repositories with per-repository access control. This guide will guide you to choose and deploy the bricks of the architecture to fit your need. 

GAALAFIS uses some well known tools and bricks when they exists, and implement a few missing ones. It also includes this guide to help you deploy the architecture.

## Why should I use GAALAFIS?

GAALAFIS is not a fully feature git product you can use out of the box. For instance, there is no frontend to manage users and repositories, to perform pull requests, and so on. It is design as the lower layer that will allow you to build such tools upon it, without having to worry about the git and git-lfs protocol, and the authentication and authorization mechanisms. 

If your goal is just to self-host your git repositories, you may want to use a more complete solution, such as [Gitea](https://gitea.io/), [Gitlab](https://about.gitlab.com/), or [Gogs](https://gogs.io/).

But if you are developing a tool that need to interact with git repositories, GAALAFIS may be a good fit for you. For instance, you may want to build a tool to manage your git repositories, or a tool to perform pull requests, or a tool to manage your CI/CD pipelines. In this case, GAALAFIS will provide you a solid base to build your tool upon.

## Features

The architecture described in this guide will provide following features:

- SSH public key authentication
- Repository access control, Read/Write access
- Git server
- Git lfs server
- Storage of lfs files on S3/MinIO
- Git lfs locks support
- Storage of locks in a PostgreSQL database

We will also guide you to deploy all the components in a docker-compose environment.

## Architecture

The architecture is composed of the following components:

- A gitolite server, that handle git repositories and access control
- A git-lfs server, that handle git-lfs objects and locks requests
- A gitolite custom command, that handle signature of a token to authorize requests to the git-lfs server
- A PostgreSQL database, that store git-lfs locks
- A MinIO server, that store git-lfs objects
- An nginx configuration, that handle the reverse proxying of the git-lfs servers

## The gitolite server

Gitolite is an opensource project, that allow you to manage git repositories and access control. It requires an admin public key to start. Then it creates a gitolite-admin repository with the following content: 

```
.
├── conf
│   └── gitolite.conf
└── keydir
    └── admin-tester.pub
```

The `conf/gitolite.conf` file contains the list of repositories and their access control. The `keydir` directory contains the public keys of the users that will be allowed to access the gitolite server.

### SSH 

With gitolite, when a user perform a git command, he does it through ssh to the git user: `git clone git@mydomain.com`. He gets an access to a limited shell, that can only execute git commands, and custom added commands (see next section). 

To deploy this component, you will need to make the port 22 available. In most cases it is already reserved to access your server. The first step is to change it. By the way it is a good practice not to use port 22 as the administration ssh port on your servers. 

- Choose a new non priviledged tcp port (> 1024), let say 1234
- Allow it in your firewall (if you have one)
- Update the file /etc/ssh/sshd_config to change the port
- Restart sshd: `sudo systemctl restart ssh`
- You should now be able to access your server with this port: `ssh me@mydomain.com -p 1234`

Alternatively, you can keep port 22 for your admin access, and change the port of the gitolite component, but then your users will need to change their git configuration to specify the port of your server

### The gitolite.conf file

The gitolite.conf file is the main configuration file of gitolite. It contains the list of repositories and their access control. It is a simple text file, with the following content

```
repo gitolite-admin
    RW+     =   admin-tester

repo testing
    RW+     =   @all
```

Here, only the admin-tester is allowed to access the gitolite-admin repository, and all users are allowed to access the testing repository. The users are identified by the name of the public key file in the keydir directory.

### The git-lfs-authenticate command

To use gitolite as the single source of truth, but without having the git-lfs server to connect to the gitolite server directly, the gitolite server will sign a token to the authenticated user, when it run `ssh git@gitolite-server git-lfs-authenticate <repo> <action>`. The git-lfs server will then verify the token, and if valid, will perform the action on the repo.

This process is done automatically by the git-lfs client, when the user run `git lfs <action> <repo>`. The git-lfs client will first connect to the git server, and ask for a token. The git server will then connect to the gitolite server, and ask for a token. The gitolite server will verify that the user can perform the action on the repo, and will sign a token. The git server will then send the token to the git-lfs client, that will send it to the git-lfs server along the request.

### Deployment

To ease the deployment of gitolite, we provide a docker image, that will init a gitolite server with the admin public key provided in the `SSH_KEY_FILE` environment variable. The gitolite server via ssh access will be available on port 22.

```yaml
services:
  gitolite:
    image: epigrapho/gaalafis-lfs-auth:0.3.20
    environment:
      BASE_URL: https://lfs.mydomain.com/
      JWT_SECRET_FILE: /run/secrets/jwt_secret
      SSH_KEY_FILE: /run/secrets/admin-tester.pub
      SSH_KEY_NAME: admin-tester
    secrets:
      - admin-tester.pub
      - jwt_secret
    volumes:
      - git_keys:/etc/ssh/keys
      - git_data:/var/lib/git
    ports:
      - "22:22"


secrets:
  admin-tester.pub:
    file: ./secrets/id_rsa.pub
  jwt_secret:
    file: ./secrets/jwt_secret
  
volumes: 
  git_data:
  git_keys:
```

- The `BASE_URL` environment variable is used to build the url of the git-lfs server. We will discuss this when we will deploy the reverse proxy.
- The `JWT_SECRET_FILE` environment variable is used to provide the secret key used to sign the jwt token. This should be a file containing a long random string. Warning: if you change this value, all the tokens will be invalidated. Also, make sure you don't have a trailing newline in the file. You can run `openssl rand -base64 64 | tr -d '\t\n ' > test` for instance to generate a 64 bytes random string, with no trailing newline.
- The `SSH_KEY_FILE` environment variable is used to provide the public key of the admin user. This should be a file containing the public key of the admin user. Copy the public key of the administrator and reference it in the `docker-compose.yaml` file.
- The `SSH_KEY_NAME` environment variable allow you to choose the name of the admin user. It will rename the public key to match this name in the keydir directory.

## MinIO

To start a MinIO server, you can use the following docker-compose service:

```yaml
services:
  bucket:
    image: minio/minio
    entrypoint: sh
    command: -c 'mkdir -p /data/bucket && minio server /data --console-address ":9001"'
    environment:
      MINIO_ACCESS_KEY_FILE: /run/secrets/sbs_access_key
      MINIO_SECRET_KEY_FILE: /run/secrets/sbs_secret_key
    volumes:
      - minio:/data
    secrets:
      - sbs_access_key
      - sbs_secret_key

secrets:
  sbs_access_key:
    file: ./secrets/sbs_access_key
  sbs_secret_key:
    file: ./secrets/sbs_secret_key
  
volumes: 
  minio:
```

The command create an initial bucket named "bucket" and start the minio server. The minio server will be available on port 9000, but here, we do not expose the port, because we will connect to minio only from other containers, and not from the host.

## PostgreSQL

To start a PostgreSQL server, you can use the following docker-compose service:

```yaml
services:
  database:
    image: postgres:15.4
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD_FILE: /run/secrets/db_password
      POSTGRES_DB: locks_db
    secrets:
      - db_password
    volumes:
      - ./postgres:/docker-entrypoint-initdb.d
      - locks_data:/var/lib/postgresql/data

secrets:
  db_password:
    file: ./secrets/db_password  
  
volumes: 
  locks_data:
```

Quite similarly, we use the official image, create the database `locks_db`. You will need to add a file `db.sql` in the `./postgres` folder, with the following content:

```sql
CREATE TABLE locks (
	id SERIAL PRIMARY KEY,
	path TEXT NOT NULL,
	ref_name TEXT NOT NULL,
	repo TEXT NOT NULL,
	owner TEXT NOT NULL,
	locked_at TIMESTAMP NOT NULL DEFAULT NOW()
);
```

## LFS server

It's now time to add the lfs server to the services. 

```yaml
services:
  lfs: 
    image: epigrapho/gaalafis-lfs-server:0.3.20-proxy-sbs-locks-pg
    environment:
      SBS_BUCKET_NAME: bucket
      SBS_ACCESS_KEY_FILE: /run/secrets/sbs_access_key
      SBS_SECRET_KEY_FILE: /run/secrets/sbs_secret_key
      SBS_REGION: us-east-1
      SBS_HOST: http://bucket:9000
      CUSTOM_SIGNER_SECRET_FILE: /run/secrets/jwt_secret
      CUSTOM_SIGNER_HOST: https://lfs.mydomain.com
      CUSTOM_SIGNER_EXPIRES_IN: 3600
      DATABASE_HOST: database
      DATABASE_USER: postgres
      DATABASE_PASSWORD_FILE: /run/secrets/db_password
      DATABASE_NAME: locks_db
      JWT_SECRET_FILE: /run/secrets/jwt_secret
      JWT_EXPIRES_IN: 3600
    ports:
      - "3000:3000"
    depends_on:
      - bucket
      - database
    secrets:
      - sbs_access_key
      - sbs_secret_key
      - jwt_secret
      - db_password

secrets:
  jwt_secret:
    file: ./secrets/jwt_secret
  sbs_access_key:
    file: ./secrets/sbs_access_key
  sbs_secret_key:
    file: ./secrets/sbs_secret_key
  db_password:
    file: ./secrets/db_password  
```

Here there are some configuration to setup: 

- SBS stands for "Single bucket storage" and is the configuration of your bucket. You can enter here your access to S3 bucket, or specify the other service.
  - SBS_BUCKET_NAME: the name of the S3/Minio bucket
  - SBS_ACCESS_KEY_FILE: a file in the container containing the access key (use secrets)
  - SBS_SECRET_KEY_FILE: a file in the container containing the secret key (use secrets)
  - SBS_REGION: if needed, specify the region of your bucket. If you use a local instance of MinIO, `us-east-1` is safe to use as a placeholder
  - SBS_HOST: the url of the bucket
- CUSTOM_SIGNER: configuration of the component signing links to upload/download files
  - CUSTOM_SIGNER_SECRET_FILE: a file in the container with the secret to sign the links (use secrets)
  - CUSTOM_SIGNER_EXPIRES_IN: in seconds, the validity time of the links
  - CUSTOM_SIGNER_HOST: where do the link point to: reference the link to the lfs sever here
- DATABASE: the postgres database connection configuration (HOST, USER, PASSWORD, and database name NAME)
- JWT_SECRET_FILE: a file in the container with the secret to verify the jwt signed by the gitolite component, and to sign jwt
- JWT_EXPIRES_IN: the expiration time of JWT in seconds

## Nginx

In most cases, the only thing to expose to the outside is the 22 port of the gitolite component, and the 3000 port of the lfs server.

```conf
limit_req_zone $binary_remote_addr zone=lfslimit:10m rate=5r/s;

server {
    server_name lfs.mydomain.com;
    limit_req zone=graphtosapilimit burst=20;
    limit_conn addr 5;
    limit_rate 300k;

    location / {
        rewrite ^/(.*)/objects/(.*)$ /objects/$2?repo=$1 last;
        rewrite ^/(.*)/locks/(.*)$ /locks/$2?repo=$1 last;
        rewrite ^/(.*)/locks$ /locks?repo=$1 last;
        proxy_set_header   X-Forwarded-For $remote_addr;
        proxy_set_header   Host $http_host;
        proxy_pass         "http://127.0.0.1:3000";
    }

    listen 443 ssl;
    ssl_certificate ...;
    ssl_certificate_key ...;
    include ...;
    ssl_dhparam ...;
}
```

In this nginx configuration for instance, we limit the number of requests to 5/s, the bandwidth to 300kB/s, we allow a burst of 20 requests. The LFS server is available on https at `lfs.mydomain.com`.

## Backups

For backups we suggest the use of [https://github.com/jareware/docker-volume-backup](https://github.com/jareware/docker-volume-backup) that allow for easy backup of volumes. You can configure it to backup locally, to an external S3 bucket, or via scp to another server. 

## What next

- You have a suggestion? You found a bug? Fill an issue.
- You want to contribute? Fork the project, and submit a pull request.
- You need more customization? Read our customization guide.
