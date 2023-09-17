#!/bin/sh

# POST_CREATE trigger to set the default branch for a new GitLab repository
# ----------------------------------------------------------------------

# Log file path
LOG_FILE="/var/log/gitolite/post_create_trigger.log"

# Function to log messages to the log file
log() {
    echo "[POST_CREATE_TRIGGER] $(date '+%Y-%m-%d %H:%M:%S'): $1" >> "$LOG_FILE"
}

# Ignore events other than repository creation
if [[ $1 != "POST_CREATE" ]]; then
    log "Ignoring event: $1 $2, not a repository creation"
    exit 0
fi
if [[ $# -gt 2 ]]; then
    log "Ignoring event: $1 $2"
    log "condition: $(($# > 2)))"
    log "reason: received $# arguments that is > 2"
    for i in "$@"; do
        log "argument: $i"
    done
    exit 0
fi

if [[ -d $GL_REPO_BASE/$2.git ]]; then
    cd $GL_REPO_BASE/$2.git
    git symbolic-ref HEAD refs/heads/main
    log "Set default branch to 'main' for repository: $2"
else
    log "Repository not found: $2"
fi
