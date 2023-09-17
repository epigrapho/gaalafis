. ./tests/helpers.sh --source-only

# leave some time for the server to start
sleep 3

# configure git
run_with_header "git config --global user.email \"you@example.com\""
run_with_header "git config --global user.name \"Your Name\""
run_with_header "git config --global init.defaultBranch main"

# clone the gitolite-admin repository
run_with_3_retries "git clone git@gitolite:gitolite-admin"

# go into the cloned repository
run_with_header "cd gitolite-admin"
run_with_header "ls -la ."

# expect to get a folder conf, and a folder keydir
expect_folder_to_exists "conf"
expect_folder_to_exists "keydir"

# verify that the conf file contains the expected content
run_with_header "cat conf/gitolite.conf"
expect_file_to_contains "conf/gitolite.conf" "repo gitolite-admin"
expect_file_to_contains "conf/gitolite.conf" "RW+     =   admin-tester"
expect_file_to_contains "conf/gitolite.conf" "repo testing"
expect_file_to_contains "conf/gitolite.conf" "RW+     =   @all"

# add a new repository
run_with_header "echo \"\" >> conf/gitolite.conf"
run_with_header "echo \"repo test-repo\" >> conf/gitolite.conf"
run_with_header "echo \"    RW+     =   admin-tester\" >> conf/gitolite.conf"
run_with_header "cat conf/gitolite.conf"
run_with_header "git status"

# commit and push
run_with_header "git add conf/gitolite.conf"
run_with_header "git status"
run_with_header "git commit -m \"add test-repo repository\""
run_with_header "git push"

# go up and clone
run_with_header "cd .."
run_with_header "git clone git@gitolite:test-repo"

# go into the cloned repository
run_with_header "cd test-repo"
run_with_header "ls -la ."

# create a new file and commit
run_with_header "echo \"test\" > test.txt"
run_with_header "git add test.txt"
run_with_header "git commit -m \"add test.txt\""
run_with_header "git rev-parse --abbrev-ref HEAD | grep main -q"
run_with_header "git push"

# go up and clone in another folder
run_with_header "cd .."
run_with_header "git clone git@gitolite:test-repo test-repo-2"

# go into the cloned repository
run_with_header "cd test-repo-2"
run_with_header "ls -la ."
expect_file_to_contains "test.txt" "test"

# verify we are on main
run_with_header "git rev-parse --abbrev-ref HEAD | grep main -q"
