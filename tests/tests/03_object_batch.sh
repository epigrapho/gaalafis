. ./tests/helpers.sh --source-only

testing_repo_upload_token=eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiI1MDAwMDAwMDAwMDAwIiwib3BlcmF0aW9uIjoidXBsb2FkIiwicmVwbyI6InRlc3RpbmciLCJ1c2VyIjoiYWRtaW4tdGVzdGVyIn0.bFZTK0MdnBlJLLkXXXKmwVMBLHSIMqeBhziVys-PBSI
testing_repo_download_token=eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiI1MDAwMDAwMDAwMDAwIiwib3BlcmF0aW9uIjoiZG93bmxvYWQiLCJyZXBvIjoidGVzdGluZyIsInVzZXIiOiJhZG1pbi10ZXN0ZXIifQ.rNfKZOwgCVN-EQj7BA1ef3q2_D-aVM2nofbEdlxPShU
testing_repo_bad_operation_token=eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiI1MDAwMDAwMDAwMDAwIiwib3BlcmF0aW9uIjoiZWxzZSIsInJlcG8iOiJ0ZXN0aW5nIiwidXNlciI6ImFkbWluLXRlc3RlciJ9.oQRjOPC5w3Cvo84aSFsmuvqT-uKsHFW7U0Mr2gQ3RHk

base_url=lfs:3000/objects/batch
upload_body="{\"operation\":\"upload\",\"transfers\":[\"basic\"],\"refs\":{\"name\":\"test\"},\"objects\":[{\"oid\":\"test2.txt\",\"size\":123}],\"hash_algo\":\"sha256\"}"
upload_body_bad_transfer="{\"operation\":\"upload\",\"transfers\":[\"unsupported\"],\"refs\":{\"name\":\"test\"},\"objects\":[{\"oid\":\"test2.txt\",\"size\":123}],\"hash_algo\":\"sha256\"}"
upload_body_bad_hash="{\"operation\":\"upload\",\"transfers\":[\"basic\"],\"refs\":{\"name\":\"test\"},\"objects\":[{\"oid\":\"test2.txt\",\"size\":123}],\"hash_algo\":\"md5\"}"
download_body="{\"operation\":\"download\",\"transfers\":[\"basic\"],\"refs\":{\"name\":\"test\"},\"objects\":[{\"oid\":\"test2.txt\",\"size\":123}],\"hash_algo\":\"sha256\"}"
download_existing_body="{\"operation\":\"download\",\"transfers\":[\"basic\"],\"refs\":{\"name\":\"test\"},\"objects\":[{\"oid\":\"test.txt\",\"size\":123}],\"hash_algo\":\"sha256\"}"

sleep 5

# 1) successful access to upload batch endpoint
run_with_header_capturing_outputs "curl -X POST '$base_url?repo=testing' -v -H 'Authorization: Bearer $testing_repo_upload_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$upload_body'"
expect_stdout_to_contain '{"transfer":"basic","objects":\[{"oid":"test2.txt","size":123,"actions":{"upload":{"href":"http://bucket:9000/bucket/testing/objects/test2.txt\?X\-Amz\-Algorithm=AWS4\-HMAC\-SHA256&X\-Amz\-Credential=.*&X\-Amz\-Date=[0-9]{8}T[0-9]{6}Z&X\-Amz\-Expires=3600&X\-Amz\-SignedHeaders=host&X\-Amz\-Signature=[a-f0-9]*","expires_in":3600}}}\],"hash_algo":"sha256"}'
expect_stderr_to_contain 'content-type: application/json'
expect_stderr_to_contain 'HTTP/1.1 200 OK'

# 2) successful access to download batch endpoint, using upload token
run_with_header_capturing_outputs "curl -X POST '$base_url?repo=testing' -v -H 'Authorization: Bearer $testing_repo_upload_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$download_existing_body'"
expect_stdout_to_contain '{"transfer":"basic","objects":\[{"oid":"test.txt","size":123,"actions":{"download":{"href":"http://bucket:9000/bucket/testing/objects/test.txt\?X\-Amz\-Algorithm=AWS4\-HMAC\-SHA256&X\-Amz\-Credential=.*&X\-Amz\-Date=[0-9]{8}T[0-9]{6}Z&X\-Amz\-Expires=3600&X\-Amz\-SignedHeaders=host&X\-Amz\-Signature=[a-f0-9]*","expires_in":3600}}}\],"hash_algo":"sha256"}'
expect_stderr_to_contain 'content-type: application/json'
expect_stderr_to_contain 'HTTP/1.1 200 OK'

# 3) successful access to download batch endpoint, using download token
run_with_header_capturing_outputs "curl -X POST '$base_url?repo=testing' -v -H 'Authorization: Bearer $testing_repo_download_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$download_existing_body'"
expect_stdout_to_contain '{"transfer":"basic","objects":\[{"oid":"test.txt","size":123,"actions":{"download":{"href":"http://bucket:9000/bucket/testing/objects/test.txt\?X\-Amz\-Algorithm=AWS4\-HMAC\-SHA256&X\-Amz\-Credential=.*&X\-Amz\-Date=[0-9]{8}T[0-9]{6}Z&X\-Amz\-Expires=3600&X\-Amz\-SignedHeaders=host&X\-Amz\-Signature=[a-f0-9]*","expires_in":3600}}}\],"hash_algo":"sha256"}'
expect_stderr_to_contain 'content-type: application/json'
expect_stderr_to_contain 'HTTP/1.1 200 OK'

# 4) access to non-existing download
run_with_header_capturing_outputs "curl -X POST '$base_url?repo=testing' -v -H 'Authorization: Bearer $testing_repo_download_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$download_body'"
expect_stdout_to_contain '{"transfer":"basic","objects":\[{"oid":"test2.txt","size":123,"error":{"message":"Not found"}}\],"hash_algo":"sha256"}'
expect_stderr_to_contain 'HTTP/1.1 200 OK'
expect_stderr_to_contain 'content-type: application/json'

# 5) upload token used to download missing file (same as download)
run_with_header_capturing_outputs "curl -X POST '$base_url?repo=testing' -v -H 'Authorization: Bearer $testing_repo_upload_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$download_body'"
expect_stdout_to_contain '{"transfer":"basic","objects":\[{"oid":"test2.txt","size":123,"error":{"message":"Not found"}}\],"hash_algo":"sha256"}'
expect_stderr_to_contain 'HTTP/1.1 200 OK'
expect_stderr_to_contain 'content-type: application/json'

# 6) download token used to upload file (unauthorized)
run_with_header_capturing_outputs "curl -X POST '$base_url?repo=testing' -v -H 'Authorization: Bearer $testing_repo_download_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$upload_body'"
expect_stdout_to_contain '{"message":"Forbidden"}'
expect_stderr_to_contain 'HTTP/1.1 403 Forbidden'
expect_stderr_to_contain 'content-type: application/json'

# 7) mismatching token repo
run_with_header_capturing_outputs "curl -X POST '$base_url?repo=other' -v -H 'Authorization: Bearer $testing_repo_upload_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$upload_body'"
expect_stdout_to_contain '{"message":"Unauthorized"}'
expect_stderr_to_contain 'HTTP/1.1 401 Unauthorized'
expect_stderr_to_contain 'content-type: application/json'

# 8) missing token
run_with_header_capturing_outputs "curl -X POST '$base_url?repo=testing' -v -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$upload_body'"
expect_stdout_to_contain '{"message":"Unauthorized"}'
expect_stderr_to_contain 'HTTP/1.1 401 Unauthorized'
expect_stderr_to_contain 'content-type: application/json'

# 9) bad token
run_with_header_capturing_outputs "curl -X POST '$base_url?repo=testing' -v -H 'Authorization: Bearer $testing_repo_bad_operation_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$upload_body'"
expect_stdout_to_contain '{"message":"Unauthorized"}'
expect_stderr_to_contain 'HTTP/1.1 401 Unauthorized'
expect_stderr_to_contain 'content-type: application/json'

# 10) empty body
run_with_header_capturing_outputs "curl -X POST '$base_url?repo=testing' -v -H 'Authorization: Bearer $testing_repo_bad_operation_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '{}'"
expect_stdout_to_contain '{"message":"Failed to deserialize the JSON body into the target type: missing field `operation` at line 1 column 2"}'
expect_stderr_to_contain 'HTTP/1.1 422 Unprocessable Entity'
expect_stderr_to_contain 'content-type: application/json'

# 11) missing repo
run_with_header_capturing_outputs "curl -X POST '$base_url' -v -H 'Authorization: Bearer $testing_repo_bad_operation_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$upload_body'"
expect_stdout_to_contain '{"message":"Failed to deserialize query string: missing field `repo`"}'
expect_stderr_to_contain 'HTTP/1.1 422 Unprocessable Entity'
expect_stderr_to_contain 'content-type: application/json'

# 12) not implemented transfer
run_with_header_capturing_outputs "curl -X POST '$base_url?repo=testing' -v -H 'Authorization: Bearer $testing_repo_upload_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$upload_body_bad_hash'"
expect_stdout_to_contain '{"message":"Invalid hash algo, only sha256 is supported"}'
expect_stderr_to_contain 'HTTP/1.1 422 Unprocessable Entity'
expect_stderr_to_contain 'content-type: application/json'

# 13) not implemented transfer
run_with_header_capturing_outputs "curl -X POST '$base_url?repo=testing' -v -H 'Authorization: Bearer $testing_repo_upload_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$upload_body_bad_transfer'"
expect_stdout_to_contain '{"message":"Only basic transfer is supported"}'
expect_stderr_to_contain 'HTTP/1.1 501 Not Implemented'
expect_stderr_to_contain 'content-type: application/json'
