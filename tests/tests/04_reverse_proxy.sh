. ./tests/helpers.sh --source-only

base_url=proxy/testing/objects/batch
testing_repo_upload_token=eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiI1MDAwMDAwMDAwMDAwIiwib3BlcmF0aW9uIjoidXBsb2FkIiwicmVwbyI6InRlc3RpbmciLCJ1c2VyIjoiYWRtaW4tdGVzdGVyIn0.bFZTK0MdnBlJLLkXXXKmwVMBLHSIMqeBhziVys-PBSI
upload_body="{\"operation\":\"upload\",\"transfers\":[\"basic\"],\"refs\":{\"name\":\"test\"},\"objects\":[{\"oid\":\"test2.txt\",\"size\":123}],\"hash_algo\":\"sha256\"}"

sleep 5

# 1) successful access to batch endpoint over HTTP
run_with_header_capturing_outputs "curl -X POST 'http://$base_url' -v -H 'Authorization: Bearer $testing_repo_upload_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$upload_body'"
expect_stdout_to_contain '{"transfer":"basic","objects":\[{"oid":"test2.txt","size":123,"actions":{"upload":{"href":"http://bucket:9000/bucket/testing/objects/test2.txt\?X\-Amz\-Algorithm=AWS4\-HMAC\-SHA256&X\-Amz\-Credential=.*&X\-Amz\-Date=[0-9]{8}T[0-9]{6}Z&X\-Amz\-Expires=3600&X\-Amz\-SignedHeaders=host&X\-Amz\-Signature=[a-f0-9]*","expires_in":3600}}}\],"hash_algo":"sha256"}'
expect_stderr_to_contain 'Content-Type: application/json'
expect_stderr_to_contain 'Connected to proxy (.*) port 80'
expect_stderr_to_contain 'HTTP/1.1 200 OK'

# 2) successful access to batch endpoint over HTTPS (use --insecure, because we use self-signed certificate for testing)
run_with_header_capturing_outputs "curl -X POST 'https://$base_url' -v -H 'Authorization: Bearer $testing_repo_upload_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$upload_body' --insecure"
expect_stdout_to_contain '{"transfer":"basic","objects":\[{"oid":"test2.txt","size":123,"actions":{"upload":{"href":"http://bucket:9000/bucket/testing/objects/test2.txt\?X\-Amz\-Algorithm=AWS4\-HMAC\-SHA256&X\-Amz\-Credential=.*&X\-Amz\-Date=[0-9]{8}T[0-9]{6}Z&X\-Amz\-Expires=3600&X\-Amz\-SignedHeaders=host&X\-Amz\-Signature=[a-f0-9]*","expires_in":3600}}}\],"hash_algo":"sha256"}'
expect_stderr_to_contain 'Content-Type: application/json'
expect_stderr_to_contain 'HTTP/1.1 200 OK'
expect_stderr_to_contain 'Connected to proxy (.*) port 443'
expect_stderr_to_contain 'SSL connection using TLS'
