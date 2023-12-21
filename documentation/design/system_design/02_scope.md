# Scope of the system

As an example, the system might have the following components. Some components might be reused from existing open-source projects, while others might be developed from scratch.

- A git server, behind a ssh server
- A git-lfs server, behind a reverse proxy
- A storage server
- An authentication wrapper
- An implementation of the git-lfs-authenticate command
- A modifiable configuration of repos and user

We can draft in the next a possible architecture of the system allowing to draw boundaries between the system and the external components

![A possible simple architecture of the system, drawing boundaries between the system and the external components](../diagrams/context.png)

The fist assumption that we will make it that the git client is not part of the system, nor is the administrator system. However, interfaces to these systems shall be provided.

Other organization might only use some components, and make other external components of the system. For example, the storage server will be considered part of the system for this project, but it should be easy to replace it with another storage server, so good interfaces will be provided.

To achieve similar features, other projects bundle everything into a single monolith, such as gitlab or gitea, making it impossible to use completely different UI or system based on these. It is possible to use gitlab as a backend for an application based on a different UI, but it is complex to configure and maintain for this use case.  
