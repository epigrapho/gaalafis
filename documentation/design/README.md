# Design documentation

## Context

**Git** is a distributed version control system widely used in software development. It might be used in a fully decentralized way, but often a central server is used to synchronize the different repositories.

**Authorization and Authentication** To answer organizational needs, the central server might need to be authenticated and authorized. This is the case for example when the server is used to store proprietary code, or when the contributors have to adhere to a code of conduct and be approved by a central authority.

**Git-lfs** is an extension to git that allows to store large files in a separate storage location. It is often used to store binary files such as images, videos, or compiled code.

# The git client

The system will be accessed from the git client, as defined for the pure git commands in the git documentation, and for the git-lfs commands in the git-lfs specification. No GUI will be provided by GAALAFIS.

# What is this project about

## The original use case

This project journey was first motivated by the desire to self-host some of my projects on my own server, and automate a few processes on the top of this. I quickly realized that I would need two features that are not native to git : lfs support and authentication. That would require a few components to be assembled together, and I thought it would be a good opportunity to apply the methods and techniques learned during my software engineering program at CentralSupélec engineering school. So after a few hours of technical exploration, I decided to go by the book and start with a system analysis.

## As a starting point

Key ideas are: 

- We want to setup a server with git, git-lfs, authentication and authorization features. 
- It shall be reproducible, and might server as a starting point to anyone that would like to setup a similar system, with slightly different requirements. It is not a complete product, but a solid ground to build upon.
- It shall be reusable: if any custom component is developed, it should be possible to use them in different architectures

## Table of contents

System design 

- [Use cases](./system_design/01_use_cases.md)
- [Scope](./system_design/02_scope.md)
- [Functions](./system_design/03_functions.md)
- [Requirements](./system_design/04_requirements.md) and the [Requirements table](./system_design/04b_table.md)
- [Components choices](./system_design/05_component_choice.md)
- [LFS server](./system_design/06_lfs_server.md)

Validation and verification

- [Unit tests & Integration tests of the LFS server](./vv/01_lfs_server_tests.md)
- [E2E tests](./vv/02_e2e.md)
