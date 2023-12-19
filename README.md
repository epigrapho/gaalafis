# GAALAFIS (Git Authenticated and Authorized LArge FIle Storage) [WIP]

![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/Leonils/gaalafis/unit-tests-on-main.yml?label=tests)
[![codecov](https://codecov.io/gh/Leonils/gaalafis/graph/badge.svg?token=YCGN2KLRLB)](https://codecov.io/gh/Leonils/gaalafis)
![GitHub License](https://img.shields.io/github/license/LeoNils/gaalafis)
![GitHub tag (with filter)](https://img.shields.io/github/v/tag/Leonils/gaalafis)

A reference architecture to serve git and git-lfs repositories with per-repository access control.

## Features

- [x] Authenticated and authorized Git server (gitolite) with per-repository access control
- [x] Git-lfs server
- [x] JWT stateless authentication and authorization
- [x] LFS objects storages
    - [x] LFS single S3 bucket storage backend
    - [ ] LFS multi S3 buckets storage backend
    - [x] Local filesystem storage backend
- [x] LFS locks (opt in)
- [x] Locks storage
    - [x] Postgres locks storage
    - [ ] Redis locks storage
    - [ ] Single S3 bucket locks storage
    - [ ] Multi S3 buckets locks storage
    - [ ] Local filesystem locks storage
- [x] Proxy/signer mode
- [x] Deployment guides
- [x] Customization guides

## Guides

- A quick guide to get started and deploy GAALAFIS in minutes using docker-compose is available in the [user guide](documentation/user-guide/user-guide.md)
- An advanced guide to customize your own version of the architecture is available in the [customization guide](documentation/user-guide/customization-guide.md)

## Modelisation

As a work done during my last year at CentralSupélec, I focused on applying system modeling technics to this project. Modelisations are available as a latex project in `documentation/design`, and the pdf is available as a release.
