#!/bin/sh

# if command is sshd, set it up correctly
if [ "${1}" = 'sshd' ]; then
  set -- /usr/sbin/sshd -D

  # Setup SSH HostKeys if needed
  for algorithm in rsa dsa ecdsa ed25519
  do
    keyfile=/etc/ssh/keys/ssh_host_${algorithm}_key
    [ -f $keyfile ] || ssh-keygen -q -N '' -f $keyfile -t $algorithm
    grep -q "HostKey $keyfile" /etc/ssh/sshd_config || echo "HostKey $keyfile" >> /etc/ssh/sshd_config
  done
  # Disable unwanted authentications
  perl -i -pe 's/^#?((?!Kerberos|GSSAPI)\w*Authentication)\s.*/\1 no/; s/^(PubkeyAuthentication) no/\1 yes/' /etc/ssh/sshd_config
  # Disable sftp subsystem
  perl -i -pe 's/^(Subsystem\ssftp\s)/#\1/' /etc/ssh/sshd_config
fi

# Fix permissions at every startup
chown -R git:git ~git

# Setup gitolite admin  
if [ ! -f ~git/.ssh/authorized_keys ]; then
  if [ -n "$SSH_KEY" ]; then
    [ -n "$SSH_KEY_NAME" ] || SSH_KEY_NAME=admin
    echo "$SSH_KEY" > "/tmp/$SSH_KEY_NAME.pub"
    su - git -c "gitolite setup -pk \"/tmp/$SSH_KEY_NAME.pub\""
    rm "/tmp/$SSH_KEY_NAME.pub"

  # else if $SSH_KEY_FILE is set, use it
  elif [ -n "$SSH_KEY_FILE" ]; then
    [ -n "$SSH_KEY_NAME" ] || SSH_KEY_NAME=admin
    su - git -c "gitolite setup -pk \"$SSH_KEY_FILE\""

  # else, we can't setup gitolite
  else 
    echo "You need to specify SSH_KEY on first run to setup gitolite"
    echo "You can also use SSH_KEY_NAME to specify the key name (optional)"
    echo 'Example: docker run -e SSH_KEY="$(cat ~/.ssh/id_rsa.pub)" -e SSH_KEY_NAME="$(whoami)" jgiannuzzi/gitolite'
    exit 1
  fi
# Check setup at every startup
else
  su - git -c "gitolite setup"
fi

# If not already uncommented, make sure LOCAL_CODE is ~/local
mkdir -p '/var/lib/git/local/triggers'
mkdir -p '/var/lib/git/local/commands'
sed -i 's/# LOCAL_CODE                =>  "$ENV{HOME}\/local/LOCAL_CODE                =>  "$ENV{HOME}\/local/' /var/lib/git/.gitolite.rc

# Add the post create trigger
if ! grep -q 'POST_CREATE => \[' /var/lib/git/.gitolite.rc; then
    sed -i '/ENABLE =>/i \    POST_CREATE => [ '\''set-head.sh'\'' ],' /var/lib/git/.gitolite.rc
fi

# Add the command git-lfs-authenticate if it doesn't exist
if ! grep -q 'git-lfs-authenticate' /var/lib/git/.gitolite.rc; then
    sed -i '/ENABLE =>/s/\[/\[ "git-lfs-authenticate", /' /var/lib/git/.gitolite.rc
fi

# Copy the implementation files
cp '/set-head.sh' '/var/lib/git/local/triggers/set-head.sh'
cp '/git-lfs-authenticate' '/var/lib/git/local/commands/git-lfs-authenticate'
cp '/.env' '/var/lib/git/local/commands/.env'

exec "$@"
