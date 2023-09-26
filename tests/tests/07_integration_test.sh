. ./tests/helpers.sh --source-only

sleep 3

# Push over lfs
run_with_header "git config --global user.email \"you@example.com\""
run_with_header "git config --global user.name \"Your Name\""
run_with_header "git config --global init.defaultBranch master"
run_with_header "git config --global http.sslverify false"
run_with_header "git clone git@gitolite:testing"
run_with_header "cd testing"
run_with_header "git lfs track '*.bin'"
run_with_header "git add .gitattributes"
run_with_header "echo \"test\" > test.bin"
run_with_header "git add test.bin"
run_with_header "git status"
run_with_header "git commit -m \"add test.bin\""
run_with_header "git push"

# The object id only depends on the file
run_with_header "git lfs pointer --file=test.bin | grep -q \"oid sha256:f2ca1bb6c7e907d06dafe4687e579fce76b37e4e93b7605022da52e6ccc26fd2\""

# manual download to verify we can get the file
base_url=proxy/testing/objects/batch
token=eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiI1MDAwMDAwMDAwMDAwIiwib3BlcmF0aW9uIjoidXBsb2FkIiwicmVwbyI6InRlc3RpbmciLCJ1c2VyIjoiYWRtaW4tdGVzdGVyIn0.bFZTK0MdnBlJLLkXXXKmwVMBLHSIMqeBhziVys-PBSI
body="{\"operation\":\"download\",\"transfers\":[\"basic\"],\"refs\":{\"name\":\"test\"},\"objects\":[{\"oid\":\"f2ca1bb6c7e907d06dafe4687e579fce76b37e4e93b7605022da52e6ccc26fd2\",\"size\":5}],\"hash_algo\":\"sha256\"}"

# Get a signed url to download the file
run_with_header_capturing_outputs "curl -X POST 'http://$base_url' -v -H 'Authorization: Bearer $token' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$body' --insecure"

# Download the file and verify that it is the same
href=$(echo "$stdout_var" | jq -r '.objects[0].actions.download.href')
run_with_header_capturing_outputs "curl '$href' -v --insecure"
expect_stdout_to_contain 'test'
expect_stderr_to_contain 'HTTP/1.1 200 OK'
expect_stderr_to_contain 'Content-Length: 5'
expect_stderr_to_contain 'Content-Type: text/plain'

# Do another clone of the repo
run_with_header "cd .."
run_with_header "git clone git@gitolite:testing testing2"
run_with_header "cd testing2"
expect_file_to_contains "test.bin" "test"

