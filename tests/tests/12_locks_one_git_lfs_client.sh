. ./tests/helpers.sh --source-only

sleep 3

# Push over lfs
run_with_header "git config --global user.email \"you@example.com\""
run_with_header "git config --global user.name \"Your Name\""
run_with_header "git config --global init.defaultBranch master"
run_with_header "git config --global http.sslverify false"
run_with_header "git config --global lfs.locksverify true"
run_with_header "git clone git@gitolite:testing"
run_with_header "cd testing"
run_with_header "git lfs track '*.bin'"
run_with_header "git add .gitattributes"
run_with_header "echo \"test\" > test.bin"
run_with_header "git add test.bin"
run_with_header "git commit -m \"add test.bin\""
run_with_header "git push"

# Lock the file
run_with_header "git lfs lock test.bin"

# Lock the file again
expect_to_fail "git lfs lock test.bin"

# List locks
run_with_header_capturing_outputs "git lfs locks"
expect_stdout_to_contain "test\.bin"
expect_stdout_to_contain "admin-tester"
expect_stdout_to_contain "ID:1"

# Unlock the file
run_with_header "git lfs unlock test.bin"

# List locks
run_with_header_capturing_outputs "git lfs locks"
expect_stdout_to_be_empty

# Create 2 locks
run_with_header "echo \"test\" > test2.bin"
run_with_header "git lfs lock test.bin"
run_with_header "git lfs lock test2.bin"

# List all locks
run_with_header_capturing_outputs "git lfs locks"
expect_stdout_to_contain "test\.bin.*admin-tester.*ID:2"
expect_stdout_to_contain "test2\.bin.*admin-tester.*ID:3"
expect_stdout_to_match_ntimes "ID:" 2

# List locks with id 1
run_with_header_capturing_outputs "git lfs locks --id=2"
expect_stdout_to_contain "test\.bin.*admin-tester.*ID:2"
expect_stdout_to_match_ntimes "ID:" 1

# List locks with path test.bin
run_with_header_capturing_outputs "git lfs locks --path=test.bin"
expect_stdout_to_contain "test\.bin.*admin-tester.*ID:2"
expect_stdout_to_match_ntimes "ID:" 1

# Create 2 more locks
run_with_header "echo \"test\" > test3.bin"
run_with_header "echo \"test\" > test4.bin"
run_with_header "git lfs lock test3.bin"
run_with_header "git lfs lock test4.bin"

# List locks with pagination (no --cursor option in the client yet)
run_with_header_capturing_outputs "git lfs locks --limit=3"
expect_stdout_to_contain "test\.bin.*admin-tester.*ID:2"
expect_stdout_to_contain "test2\.bin.*admin-tester.*ID:3"
expect_stdout_to_contain "test3\.bin.*admin-tester.*ID:4"
expect_stdout_to_match_ntimes "ID:" 3
