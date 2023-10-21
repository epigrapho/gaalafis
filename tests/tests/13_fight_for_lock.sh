. ./tests/helpers.sh --source-only

sleep 3

# Push over lfs
run_with_header "git config --global user.email \"you@example.com\""
run_with_header "git config --global user.name \"Your Name\""
run_with_header "git config --global init.defaultBranch master"
run_with_header "git config --global http.sslverify false"
run_with_header "git config --global lfs.locksverify true"

# Act as admin-tester, add ssh key of user 2
run_with_header "cd ~"
run_with_3_retries "git clone git@gitolite:gitolite-admin"
run_with_header "cd gitolite-admin/keydir"
run_with_header "cp ~/.ssh/id_rsa_2.pub user2.pub"
run_with_header "git add user2.pub"
run_with_header "git commit -m \"Add user2 ssh key\""
run_with_header "git push"
run_with_header "cd ~"

# As admin-tester, init the lfs repo
run_with_3_retries "git clone -c core.sshCommand=\"/usr/bin/ssh -i ~/.ssh/id_rsa\" git@gitolite:testing testing1"
run_with_header "cd testing1"
run_with_header "git config --local core.sshCommand \"/usr/bin/ssh -i ~/.ssh/id_rsa\""
run_with_header "git lfs track '*.bin'"
run_with_header "git add .gitattributes"
run_with_header "echo \"test\" > test1.bin"
run_with_header "echo \"test\" > test2.bin"
run_with_header "echo \"test\" > test3.bin"
run_with_header "git add ."
run_with_header "git commit -m \"add 3 bin files\""
run_with_header "git push"
run_with_header "cd ~"

# Act as user2, add a lock on test1.bin
run_with_header "git clone -c core.sshCommand=\"/usr/bin/ssh -i ~/.ssh/id_rsa_2\" git@gitolite:testing testing2"
run_with_header "cd testing2"
run_with_header "git config --local core.sshCommand \"/usr/bin/ssh -i ~/.ssh/id_rsa_2\""
run_with_header "git lfs lock test1.bin"

# Act as admin-tester, list locks
run_with_header "cd ~/testing1"
run_with_header_capturing_outputs "git lfs locks"
expect_stdout_to_contain "test1\.bin.*user2.*ID:1"
expect_stdout_to_match_ntimes "ID:" 1

# Act as admin-tester, add a lock on test2.bin
run_with_header "git lfs lock test2.bin"
run_with_header_capturing_outputs "git lfs locks"
expect_stdout_to_contain "test1\.bin.*user2.*ID:1"
expect_stdout_to_contain "test2\.bin.*admin-tester.*ID:2"
expect_stdout_to_match_ntimes "ID:" 2

# Act as user2, try to add a lock on test2.bin
run_with_header "cd ~/testing2"
expect_to_fail "git lfs lock test2.bin"
run_with_header_capturing_outputs "git lfs locks"
expect_stdout_to_contain "test1\.bin.*user2.*ID:1"
expect_stdout_to_contain "test2\.bin.*admin-tester.*ID:2"

# Act as user2, list locks for verification
run_with_header_capturing_outputs "git lfs locks --verify"
expect_stdout_to_contain "O test1\.bin.*user2.*ID:1"
expect_stdout_to_contain "  test2\.bin.*admin-tester.*ID:2"

# Act as user2, force unlock test2.bin
run_with_header "git lfs unlock --force test2.bin"
run_with_header_capturing_outputs "git lfs locks"
expect_stdout_to_contain "test1\.bin.*user2.*ID:1"
expect_stdout_to_match_ntimes "ID:" 1

# Act as user2, set our own lock on test2.bin
run_with_header "git lfs lock test2.bin"
run_with_header_capturing_outputs "git lfs locks --verify"
expect_stdout_to_contain "O test1\.bin.*user2.*ID:1"
expect_stdout_to_contain "O test2\.bin.*user2.*ID:3"

# Act as admin-tester, create a new branch, edit test1.bin and try to push
# It shall fails, because the file is locked for modification by user 2
run_with_header "cd ~/testing1"
run_with_header "git checkout -b newbranch"
run_with_header "echo \"test updated\" > test1.bin"
run_with_header "git status"
run_with_header "git add test1.bin"
run_with_header "git commit -m \"edit test1.bin\""
expect_to_fail_capturing_output "git push origin newbranch"
expect_stdout_to_contain "Unable to push locked files"
expect_stdout_to_contain "test1.bin - user2 \(refs: newbranch\)"
expect_stderr_to_contain "Cannot update locked files"
expect_stderr_to_contain "error: failed to push some refs to 'gitolite:testing'"
