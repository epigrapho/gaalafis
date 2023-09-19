. ./tests/helpers.sh --source-only

# leave some time for the server to start
sleep 3

# run git-lfs-authenticate
run_with_3_retries "ssh git@gitolite git-lfs-authenticate testing upload"

# An optional last argument can be specified (not used for now)
run_with_3_retries "ssh git@gitolite git-lfs-authenticate testing upload ref"

# for invalid inputs, we should get an error
expect_to_fail "ssh git@gitolite git-lfs-authenticate testing invalid"
expect_to_fail "ssh git@gitolite git-lfs-authenticate testing ref too_many"
expect_to_fail "ssh git@gitolite git-lfs-authenticate unknown_repo"
