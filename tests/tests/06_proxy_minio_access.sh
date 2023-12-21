. ./tests/helpers.sh --source-only

base_url=proxy/testing/objects/batch
testing_repo_upload_token=eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiI1MDAwMDAwMDAwMDAwIiwib3BlcmF0aW9uIjoidXBsb2FkIiwicmVwbyI6InRlc3RpbmciLCJ1c2VyIjoiYWRtaW4tdGVzdGVyIn0.bFZTK0MdnBlJLLkXXXKmwVMBLHSIMqeBhziVys-PBSI
upload_body="{\"operation\":\"upload\",\"transfers\":[\"basic\"],\"refs\":{\"name\":\"test\"},\"objects\":[{\"oid\":\"test-upload.txt\",\"size\":15}],\"hash_algo\":\"sha256\"}"
download_body="{\"operation\":\"download\",\"transfers\":[\"basic\"],\"refs\":{\"name\":\"test\"},\"objects\":[{\"oid\":\"test-upload.txt\",\"size\":15}],\"hash_algo\":\"sha256\"}"

sleep 5

# Get a signed url to upload a file
run_with_header_capturing_outputs "curl -X POST 'https://$base_url' -v -H 'Authorization: Bearer $testing_repo_upload_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$upload_body' --insecure"
expect_stdout_to_contain '{"transfer":"basic","objects":\[{"oid":"test-upload.txt","size":15,"actions":{"upload":{"href":"https://proxy/bucket/testing/objects/test-upload.txt\?X\-Amz\-Algorithm=AWS4\-HMAC\-SHA256&X\-Amz\-Credential=.*&X\-Amz\-Date=[0-9]{8}T[0-9]{6}Z&X\-Amz\-Expires=3600&X\-Amz\-SignedHeaders=host&X\-Amz\-Signature=[a-f0-9]*","expires_in":3600}}}\],"hash_algo":"sha256"}'

# upload file
echo "Test of upload." > /tmp/test-upload.txt
href=$(echo "$stdout_var" | jq -r '.objects[0].actions.upload.href')
run_with_header_capturing_outputs "curl -X PUT '$href' -v --insecure -H 'Content-Type: text/plain' -H 'Content-Length: 15' --data-binary @/tmp/test-upload.txt"

# get the download link to the file
run_with_header_capturing_outputs "curl -X POST 'https://$base_url' -v -H 'Authorization: Bearer $testing_repo_upload_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$download_body' --insecure"
expect_stdout_to_contain '{"transfer":"basic","objects":\[{"oid":"test-upload.txt","size":15,"actions":{"download":{"href":"https://proxy/bucket/testing/objects/test-upload.txt\?X\-Amz\-Algorithm=AWS4\-HMAC\-SHA256&X\-Amz\-Credential=.*&X\-Amz\-Date=[0-9]{8}T[0-9]{6}Z&X\-Amz\-Expires=3600&X\-Amz\-SignedHeaders=host&X\-Amz\-Signature=[a-f0-9]*","expires_in":3600}}}\],"hash_algo":"sha256"}'
href=$(echo "$stdout_var" | jq -r '.objects[0].actions.download.href')

# Download the file and verify that it is the same
run_with_header_capturing_outputs "curl '$href' -v --insecure"
expect_stdout_to_contain 'Test of upload.'
expect_stderr_to_contain 'HTTP/1.1 404 OK'
expect_stderr_to_contain 'Host: proxy'
expect_stderr_to_contain 'Content-Length: 15'
expect_stderr_to_contain 'Content-Type: text/plain'

