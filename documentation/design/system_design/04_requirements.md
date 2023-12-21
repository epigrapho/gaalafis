# Requirements

Last sections brought up to 19 requirements in total. Let's try to organize them in a more meaningful way.

At the highest level, we can identify four main requirements:

- **R1**: The system shall implement git and git-lfs repositories over SSH.
- **R2**: The system shall be usable as a low-level foundation for commercial and non-commercial multi-users applications based around git repositories.
- **R3**: The system shall be developed stable, maintainable, and documented.
- **R4**: The system shall be deployable in less than 1h in an enterprise-grade environment.

## R1: git and git-lfs repositories over SSH.

Here are the requirements that were identified in the previous sections, related to R1.

- **R1.1**: The system shall serve all regular git commands (clone, push, pull, etc.).
- **R1.2**: The system shall allow for SSH git authentication.
- **R1.3**: The system shall implement the git-lfs batch/objects API.
- **R1.4**: The system shall implement the git-lfs locks API.

## R2: The a low-level foundation for commercial and non-commercial multi-users applications based around git repositories.

From the start, I intended to use the system as a basis for other projects, such as a git-based wiki, or a git-based issue tracker. Many applications might benefit from the versioning API of git, and lack of support of file storage is often a deal-breaker to build such apps. By bringing a clear and lightweight foundation for such applications, I hope to make it easier to build them, and to make them more robust.

But to achieve this, the system must verify a strong set of constraints to be extended safely, legally, and to be used in production: 

- **R2.1**: The system shall allow for per-repository access control, including read and write access.
- **R2.2**: The system shall store LFS files in a separate storage location.
- **R2.3**: Components of the system shall be reusable in other systems.
- **R2.4**: The system shall be configurable by a single administrator actor (other authorized system or actual person).
- **R2.5**: The system shall use compatible licences, and be loosely coupled enough so the licences of components do not contaminate each other. 
- **R2.6**: The system shall not use deprecated components.
- **R2.7**: Abstraction done of the network layer between the client and the system, the system shall handle an initial push of a 100MB of LFS as 100 large files in less than one minute.

## R3. Maintainability. 

From a personal perspective, I want to be able to develop the system in a reasonable amount of time, and then to have minimal needs for maintenance. To achieve this, the project must ensure correctness as much as possible, and to keep it simple. To achieve this, I will use the following requirements:

- **R3.1**: The system shall ensure memory safety and functional correctness either by being heavily maintained by the community, by formal technics, or by automated testing. 
- **R3.2**: Every custom code shall adhere to a common and well-recognized good practices handbook, to be chosen. 
- **R3.3**: The system shall be documented focussing on: the system in its whole, the system components, maintenance processes. 

## R4: deployable in less than 1h in an enterprise-grade environment.

Once the project is ready, it must be easy to deploy in a production environment. One big source of complexity is the state management. A pure function has no complexity at all. At the opposite, we can often implement caching to improve time performance, but it comes at the cost of complexity and memory performance. To keep the system simple and efficient, we will make it as stateless as possible. Of course data will be kept, but infrastructure, tokens, ... will be kept in a stateless way. Conteneurisation is a good way to achieve this, as the code describes the architecture, that can be deployed at any scale. 

Being enterprise friendly also means providing high level of security and control. 

These considerations lead to the following requirements:

- **R4.1**: The system shall be deployable in conteneurised linux environments in a reproducible and documented way.  
- **R4.2**: Only the lock API, the batch API, and the git commands shall be exposed to the outside.
- **R4.3**: The authorization systems shall not share state between the git server and the git-lfs server or the storage server.
- **R4.4**: The administrator of the system shall be able to define resource limits for the bandwidth, the storage, and the number of requests, separated between the target points (lfs, git), and discriminated by user and repository.
- **R4.5**: The system shall do regular backups of the data.

## Stateless proof of access

**R4.3** is implemented by two different mechanisms:

- Between the authentication components and the lfs info server, the proof of access is implemented by a json web token, including the user logical name, the repository name, the operation requested (download/upload) and the expiration date.
- Between the time the user announce he want to send or get files, and the several requests he makes to get or send them, the proof of access is implemented by a signed link given by the lfs info server and verified when the user tries to send or get files.

Deriving from this and from the git lfs server api specification, **R4.3** and **R2.1** can be further decomposed into:

- **R2.1.1**: The authentication and authorization subsystem shall be able to verify if a given user has read or write access to a given repository.
- **R2.1.2**: The authentication and authorization subsystem shall expose a command named *git-lfs-authenticate*, which will return a json web token signing the user, the repository, the operation, and the expiration date.
- **R2.1.3**: The lfs server shall accept and verify a json web token signing the user, the repository, the operation, and the expiration date.
- **R2.1.4**: The lfs server shall be able to generate a signed link for a given file, operation, and expiration date that can't be modified by the user.
- **R2.1.5**: The file server (either the lsf info server acting as a proxy or the storage server directly) shall be able to verify a signed link for a given file, operation, and expiration date, and serve or accept the requested file, depending on the operation. 

## R1.3 and R1.4: Implementation of the lfs API

The lfs apis are quite well defined by their respective specifications, and can be derived later as needed.

## Reusability of the system with other file storages

Deriving from **R2.2** and **R2.3**, the system shall implement a set of interfaces to be able to use other file storages. Actually, not all file storages will be implemented, but at least two will be, so we focus on the interfaces rather than the implementations.

As a priority for the first implementation, the system will implement the MinIO API, and the local file system API. The MinIO API is a S3 compatible API. Two policy are possible for the MinIO storage: one bucket per repository, or one bucket for all repositories. We will focus first on the single bucket policy. Following requirements are derived from this:

- **R2.3.1**: The system shall define file storage interfaces, to retrieve metadata about a file, to retrieve a file, and to store a file.
- **R2.3.2**: The system shall model implementations of the storage interfaces of **R2.3.1** for other storages: Multiple Bucket Storage policy (MBS), and File Transfer Protocol (FTP), and local storage with database stored metadata (LSDB)
- **R2.2.1**: The system shall implement the storage interfaces of **R2.3.1** for the MinIO API, using a Single Bucket Storage policy (SBS)
- **R2.2.2**: The system shall implement the storage interfaces of **R2.3.1** for the local file system API

On the other side, locks have to be stored too. In the same way, the system must define interfaces, provide a few implementations, and model future ones: 

- **R2.3.3**: The system shall define lock storage interfaces, to create, list, and delete locks.
- **R2.3.4**: The system shall model implementations of the lock storage interfaces of **R2.3.3** for other storages: Locks in multiple buckets (LMB), Locks in local file storage.
- **R2.2.3**: The system shall implement the lock storage interfaces of **R2.3.3** for the MinIO API, using a Single Bucket Storage policy (SBS)
- **R2.2.4**: The system shall implement the lock storage interfaces of **R2.3.3** for postgresql database

## Software quality

Deriving from **R3.2**, **R3.3**, and **R3.4**, the system shall focus on quality. Several ways can be used. The first one is the use formal methods: the use of the rust programming language, focussing on memory safety and a strong type system will help to achieve this. The second one is the use of automated testing. The third one is the use of a common good practices handbook. The fourth one is the use of documentation. This defines the following requirements:

- **R3.2.1**: Custom parts of the system shall be implemented in the rust programming language.
- **R3.2.2**: Non custom parts of the system shall be heavily adopted by the community to be considered safe.
- **R3.2.3**: Custom parts of the system shall be tested using automated testing, with a coverage of at least 90\%.
- **R3.3.1**: Custom apis of the system should adhere to the following good practices handbooks [https://rust-lang.github.io/api-guidelines/](https://rust-lang.github.io/api-guidelines/), [https://cheats.rs/#idiomatic-rust](https://cheats.rs/#idiomatic-rust) and [https://rust-unofficial.github.io/patterns/](https://rust-unofficial.github.io/patterns/)
- **R3.3.2**: Custom code of the system shall use the linter [https://github.com/rust-lang/rust-clippy](https://github.com/rust-lang/rust-clippy)
- **R3.4.1**: Every custom and non-custom code of the system shall be documented in term of context and role in the system, and in term of possible replacements.
- **R3.4.2**: The system shall be documented in term of architecture, and in term of maintenance processes.
- **R3.4.3**: The system shall be documented in term of deployment processes.
- **R3.4.4**: Custom code of the system shall be documented in term of technical choices and design.
