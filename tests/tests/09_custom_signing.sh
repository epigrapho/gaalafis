. ./tests/helpers.sh --source-only

random_file_name=$(cat /dev/urandom | tr -dc 'a-zA-Z0-9' | fold -w 15 | head -n 1)
base_url=proxy/testing/objects/batch
testing_repo_upload_token=eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiI1MDAwMDAwMDAwMDAwIiwib3BlcmF0aW9uIjoidXBsb2FkIiwicmVwbyI6InRlc3RpbmciLCJ1c2VyIjoiYWRtaW4tdGVzdGVyIn0.bFZTK0MdnBlJLLkXXXKmwVMBLHSIMqeBhziVys-PBSI
upload_body="{\"operation\":\"upload\",\"transfers\":[\"basic\"],\"refs\":{\"name\":\"test\"},\"objects\":[{\"oid\":\"$random_file_name.txt\",\"size\":15}],\"hash_algo\":\"sha256\"}"
download_body="{\"operation\":\"download\",\"transfers\":[\"basic\"],\"refs\":{\"name\":\"test\"},\"objects\":[{\"oid\":\"$random_file_name.txt\",\"size\":15}],\"hash_algo\":\"sha256\"}"

sleep 5

# Get a signed url to upload a file
run_with_header_capturing_outputs "curl -X POST 'https://$base_url' -v -H 'Authorization: Bearer $testing_repo_upload_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$upload_body' --insecure"
expect_stdout_to_contain "{\"transfer\":\"basic\",\"objects\":\[{\"oid\":\"$random_file_name.txt\",\"size\":15,\"actions\":{\"upload\":{\"href\":\"https://proxy/testing/objects/access/$random_file_name.txt\",\"header\":{\"Authorization\":\"Bearer .*\..*\..*\"},\"expires_in\":3600}}}\],\"hash_algo\":\"sha256\"}"

# upload file
echo "Test of upload." > /tmp/$random_file_name.txt
href=$(echo "$stdout_var" | jq -r '.objects[0].actions.upload.href')
token=$(echo "$stdout_var" | jq -r '.objects[0].actions.upload.header.Authorization')
run_with_header_capturing_outputs "curl -X PUT '$href' -v --insecure -H 'Content-Type: text/plain' -H 'Content-Length: 15' -H 'Authorization: $token' --data-binary @/tmp/$random_file_name.txt"
expect_stderr_to_contain 'HTTP/1.1 200 OK'

# get the download link to the file
run_with_header_capturing_outputs "curl -X POST 'https://$base_url' -v -H 'Authorization: Bearer $testing_repo_upload_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$download_body' --insecure"
expect_stdout_to_contain "{\"transfer\":\"basic\",\"objects\":\[{\"oid\":\"$random_file_name.txt\",\"size\":15,\"actions\":{\"download\":{\"href\":\"https://proxy/testing/objects/access/$random_file_name.txt\",\"header\":{\"Authorization\":\"Bearer .*\..*\..*\"},\"expires_in\":3600}}}\],\"hash_algo\":\"sha256\"}"
href=$(echo "$stdout_var" | jq -r '.objects[0].actions.download.href')

# # Download the file and verify that it is the same
run_with_header_capturing_outputs "curl '$href' -v --insecure -H 'Authorization: $token'"
expect_stdout_to_contain 'Test of upload.'
expect_stderr_to_contain 'HTTP/1.1 200 OK'
expect_stderr_to_contain 'Host: proxy'
expect_stderr_to_contain 'Content-Length: 15'
expect_stderr_to_contain 'Content-Type: text/plain'
