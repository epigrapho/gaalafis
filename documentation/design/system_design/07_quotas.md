# Usage quota

The system shall implement limits and monitoring of space used by each repository. There is 3 types of data stored in the system: The locks, the LFS objects, and the repository regular objects. 

**The locks** can be easily counted and monitored in the locks controller. They should not be the major source of data usage, so it makes little sense to have a precise configuration for them. We will simply implement a common limit to the number of locks a repository can hold at the same time to prevent manifest abuse. 

**The number of repository** shall be limited by administrator of repositories.

The sizes of the objects on the other hand might need to be configured on a per-repo basis. For that we will use gitolite options to define environment variables that can define `REPO_MAX_SIZE`, `LFS_REPO_MAX_SIZE`. These can be defaulted in the LFS server and gitolite image config.

The repo max size will be enforced by a custom gitolite update VREF [See docs](https://gitolite.com/gitolite/vref.html). When a push is made against the gitolite server, the current size of the repository is added with the size of the diff to approximate the resulting repository size and prevent the push if it go beyond the limit.

The lfs repo max size will need some more work. First, the `git-lfs-authenticate` will sign the `LFS_REPO_MAX_SIZE` value. The LFS server will then be able to decode it. Then according to the implementation, it might differ: 

### The proxy implementation

When using the proxy variant, every upload pass through the LFS server. It is then easy for the server to count the incoming bytes and sum them up in a database. We need to anticipate what's happen if the upload partially fails at the end of it. If we update the size before the upload, it will result in over-evaluation of the use size. If we set it after, in case of late failure, it might not be updated and so we might under-evaluate the use of the server, that is a greater issue. 

### The non-proxy implementation

When not using the proxy, only links to upload files are signed by the LFS server, and then another target (a S3 bucket for instance) handle the upload. If it can't handle the usage limit, enforcing it is complex: If we count usage based on links used, unused links will be counted too, and the use might be largely over-estimated. We might verify it by regularly checking the actual usage on the bucket, but that come at a cost. A better approach would be to track generated links on the LFS server and regularly, check for all generated links that are expired, try to retrieve the file on the bucket. If the file is present, we can validate the usage and add it to the final total. If not, we can discard it, as the link will not be used (expired). The push is blocked if the tracked pending links plus the grand total are larger than the limit.

This scheme is more complexe but have the advantage of being eventually consistant between the bucket and the LFS server with minimal data exchange (only one read per object at the correct time). It might result in a false positive deny of a push if the user generates a lot of unused links, but after their expiration, if not used, he will be able to push again.

### What to implement

- [ ] Locks number limitation
- [ ] Gitolite VREF to limit the repo size
- [ ] A gitolite command to get the current usage of the repo
- [ ] A route on the LFS server to get the current usage of lfs of a repo
- [ ] Adapt the `git-lfs-authenticate` to specify the allowed total usage
- [ ] Non-proxy post-upload hook
- [ ] Proxy post-link hook + cron job to rectify usage
