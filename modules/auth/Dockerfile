FROM alpine:3.10

# Install gitolite (to add authorization layer over git)
# Install openssh (to be able to clone over SSH)
# Install apache (to serve lfs server)
# Unlock automatically created user git
RUN set -x && apk add --no-cache gitolite openssh && passwd -u git

# Volume to store host keys (generated on first run)
VOLUME /etc/ssh/keys

# Volume to store all gitolite data from one run to the other
VOLUME /var/lib/git

# Logs directory owned by gitolite
RUN mkdir -p /var/log/gitolite && chown git:git /var/log/gitolite

# Set head to main trigger
COPY scripts/set-head.sh /
RUN chmod +x 'set-head.sh'

# Entry point
COPY scripts/docker-entrypoint.sh /
RUN chmod +x docker-entrypoint.sh
ENTRYPOINT ["/docker-entrypoint.sh"]

# Expose port 22 for SSH access
EXPOSE 22

# Start SSH server and apache server
CMD ["sshd"]
