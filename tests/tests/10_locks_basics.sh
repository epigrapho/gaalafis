. ./tests/helpers.sh --source-only

base_url=proxy/testing/locks
testing_repo_upload_token=eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiI1MDAwMDAwMDAwMDAwIiwib3BlcmF0aW9uIjoidXBsb2FkIiwicmVwbyI6InRlc3RpbmciLCJ1c2VyIjoiYWRtaW4tdGVzdGVyIn0.bFZTK0MdnBlJLLkXXXKmwVMBLHSIMqeBhziVys-PBSI
create_lock_body="{\"path\":\"foo/bar.bin\",\"ref\":{\"name\":\"master\"}}"
create_lock_body2="{\"path\":\"foo/bar2.bin\",\"ref\":{\"name\":\"master\"}}"
create_lock_body3="{\"path\":\"foo/bar3.bin\",\"ref\":{\"name\":\"master\"}}"
create_lock_body4="{\"path\":\"foo/bar4.bin\",\"ref\":{\"name\":\"master\"}}"
date_regex="[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\+[0-9]{2}:[0-9]{2}"

sleep 5

# Create lock for path foo/bar.bin
run_with_header_capturing_outputs "curl -X POST 'https://$base_url' -v -H 'Authorization: Bearer $testing_repo_upload_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$create_lock_body' --insecure"
expect_stdout_to_contain "{\"lock\":{\"id\":\"1\",\"path\":\"foo/bar\.bin\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"admin-tester\"}}}"
expect_stderr_to_contain "HTTP/1.1 201 Created"

# Create duplicate lock for path foo/bar1.bin, should fail
run_with_header_capturing_outputs "curl -X POST 'https://$base_url' -v -H 'Authorization: Bearer $testing_repo_upload_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$create_lock_body' --insecure"
expect_stdout_to_contain "{\"lock\":{\"id\":\"1\",\"path\":\"foo/bar\.bin\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"admin-tester\"}},\"message\":\"already created lock\"}"
expect_stderr_to_contain "HTTP/1.1 409 Conflict"

# Create lock for path foo/bar2.bin
run_with_header_capturing_outputs "curl -X POST 'https://$base_url' -v -H 'Authorization: Bearer $testing_repo_upload_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$create_lock_body2' --insecure"
expect_stdout_to_contain "{\"lock\":{\"id\":\"2\",\"path\":\"foo/bar2\.bin\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"admin-tester\"}}}"
expect_stderr_to_contain "HTTP/1.1 201 Created"

# List locks for path foo/bar.bin and foo/bar2.bin
run_with_header_capturing_outputs "curl -X GET 'https://$base_url?path=foo/bar.bin' -v -H 'Authorization: Bearer $testing_repo_upload_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' --insecure"
expect_stdout_to_contain "{\"locks\":\[{\"id\":\"1\",\"path\":\"foo/bar\.bin\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"admin-tester\"}}\]}"
expect_stderr_to_contain "HTTP/1.1 200 OK"

run_with_header_capturing_outputs "curl -X GET 'https://$base_url?path=foo/bar2.bin' -v -H 'Authorization: Bearer $testing_repo_upload_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' --insecure"
expect_stdout_to_contain "{\"locks\":\[{\"id\":\"2\",\"path\":\"foo/bar2\.bin\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"admin-tester\"}}\]}"
expect_stderr_to_contain "HTTP/1.1 200 OK"

# List locks by id
run_with_header_capturing_outputs "curl -X GET 'https://$base_url?id=2' -v -H 'Authorization: Bearer $testing_repo_upload_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' --insecure"
expect_stdout_to_contain "{\"locks\":\[{\"id\":\"2\",\"path\":\"foo/bar2\.bin\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"admin-tester\"}}\]}"
expect_stderr_to_contain "HTTP/1.1 200 OK"

# Empty search params should be ignored
run_with_header_capturing_outputs "curl -X GET 'https://$base_url?path=&id=&cursor=&limit=&refspec=' -v -H 'Authorization: Bearer $testing_repo_upload_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' --insecure"
expect_stdout_to_contain "{\"locks\":\[{\"id\":\"1\",\"path\":\"foo/bar\.bin\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"admin-tester\"}},{\"id\":\"2\",\"path\":\"foo/bar2\.bin\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"admin-tester\"}}\]}"
expect_stderr_to_contain "HTTP/1.1 200 OK"

# Create a few more locks for pagination
run_with_header_capturing_outputs "curl -X POST 'https://$base_url' -v -H 'Authorization: Bearer $testing_repo_upload_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$create_lock_body3' --insecure"
run_with_header_capturing_outputs "curl -X POST 'https://$base_url' -v -H 'Authorization: Bearer $testing_repo_upload_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$create_lock_body4' --insecure"

# List locks with pagination
run_with_header_capturing_outputs "curl -X GET 'https://$base_url?limit=3' -v -H 'Authorization: Bearer $testing_repo_upload_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' --insecure"
expect_stdout_to_contain "{\"locks\":\[{\"id\":\"1\",\"path\":\"foo/bar\.bin\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"admin-tester\"}},{\"id\":\"2\",\"path\":\"foo/bar2\.bin\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"admin-tester\"}},{\"id\":\"3\",\"path\":\"foo/bar3\.bin\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"admin-tester\"}}\],\"next_cursor\":\"4\"}"
expect_stderr_to_contain "HTTP/1.1 200 OK"

run_with_header_capturing_outputs "curl -X GET 'https://$base_url?limit=3&cursor=4' -v -H 'Authorization: Bearer $testing_repo_upload_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' --insecure"
expect_stdout_to_contain "{\"locks\":\[{\"id\":\"4\",\"path\":\"foo/bar4\.bin\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"admin-tester\"}}\]}"
expect_stderr_to_contain "HTTP/1.1 200 OK"

# Delete lock
run_with_header_capturing_outputs "curl -X POST 'https://$base_url/1/unlock' -v -H 'Authorization: Bearer $testing_repo_upload_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '{}' --insecure"
expect_stdout_to_contain "{\"lock\":{\"id\":\"1\",\"path\":\"foo/bar\.bin\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"admin-tester\"}}}"
expect_stderr_to_contain "HTTP/1.1 200 OK"

# Delete lock again
run_with_header_capturing_outputs "curl -X POST 'https://$base_url/1/unlock' -v -H 'Authorization: Bearer $testing_repo_upload_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '{}' --insecure"
expect_stdout_to_contain '{"message":"Not found"}'
expect_stderr_to_contain "HTTP/1.1 404 Not Found"

# Delete lock with invalid id
run_with_header_capturing_outputs "curl -X POST 'https://$base_url/abc/unlock' -v -H 'Authorization: Bearer $testing_repo_upload_token' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '{}' --insecure"
expect_stdout_to_contain '{"message":"InvalidId"}'
expect_stderr_to_contain "HTTP/1.1 422 Unprocessable Entity"
