FROM ubuntu:22.04

# install git
RUN apt-get update && apt-get install -y git

WORKDIR /root

# we are in an isolated environment, so we can disable host key checking
RUN sed -i '/Host \*/,/^\s*$/ s/#\?\(\s*\)\(StrictHostKeyChecking\s\+\).*$/\1StrictHostKeyChecking no\n\1UserKnownHostsFile \/dev\/null/' /etc/ssh/ssh_config

# copy the runner
COPY runner/run_test.sh run_test.sh
RUN chmod +x run_test.sh

# run the tests
CMD ./run_test.sh
