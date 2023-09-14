# GAALAFIS (Git Authenticated and Authorized LArge FIle Storage) [WIP]

A reference architecture to serve git and git-lfs repositories with per-repository access control.

## Reminder: how does LFS work server(s) side

### Authentication

How do other implementation handle authentication?

- [git-lfs/lfs-test-server](https://github.com/git-lfs/lfs-test-server) (The reference implementation of Git-LFS server) implement a basic authentication mechanism, with an admin interface to add users by username/password. 
- [HimaJyun/gitolite-lfs](https://github.com/HimaJyun/gitolite-lfs) make use of .htpasswd file to authenticate users
- [kzwang/node-git-lfs](https://github.com/kzwang/node-git-lfs) implement a basic user/password authentication mechanism
- [charmbracelet/git-lfs-transfer](https://github.com/charmbracelet/git-lfs-transfer) implement a proposal SSH-only protocol, but that require to have a dedicated client.
- [artemkin/git-lfs-server](https://github.com/artemkin/git-lfs-server) uses PAM to authenticate users

However, a pure stateless authentication mechanism is described in the [git-lfs specification](https://github.com/git-lfs/git-lfs/blob/main/docs/api/authentication.md). This repository intends to implement this mechanism with JWT:

Lets consider 3 actors:

- The client
- The git server, wrapped with an authentication layer
- The git-lfs server, not knowing anything about users

The dialog is the following:

- The client will first authenticate to the git server over SSH, with the "git-lfs-authenticate <repo> <action>" command.
- The git server will verify that user can do action on repo, and sign a jwt token with the claim
- The client will then send the jwt token to the git-lfs server along the download/upload request
- The git-lfs server will verify the token, and if valid, will perform the action on the repo

## LFS Storage backend

2 backend are planned to be supported:

- [ ] Local filesystem
- [ ] S3

## Modelisation

As a work done during my last year at CentralSupélec, I focused on applying system modeling technics to this project. Modelisations are available as a latex project in `documentation/design`, and the pdf is available as a release.
