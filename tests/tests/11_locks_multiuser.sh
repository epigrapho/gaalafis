. ./tests/helpers.sh --source-only

base_url=proxy/testing/locks
token_user1=eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiI1MDAwMDAwMDAwMDAwIiwib3BlcmF0aW9uIjoidXBsb2FkIiwicmVwbyI6InRlc3RpbmciLCJ1c2VyIjoidXNlcjEifQ.vGMY3IXRPcOvxu1Fbxen7b31L2jIUvv8msPU66vfL2c
token_user2=eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiI1MDAwMDAwMDAwMDAwIiwib3BlcmF0aW9uIjoidXBsb2FkIiwicmVwbyI6InRlc3RpbmciLCJ1c2VyIjoidXNlcjIifQ.I8Tg6mNdGvDd9KvyywQJsgcqzV38hk6xYhsCvfvrro8

limit3="{\"limit\":\"3\"}"
force_body="{\"force\":true}"
limit3cursor4="{\"limit\":\"3\",\"cursor\":\"4\"}"
create_lock_body1="{\"path\":\"file1\"}}"
create_lock_body2="{\"path\":\"file2\"}}"
create_lock_body3="{\"path\":\"file3\"}}"
create_lock_body4="{\"path\":\"file4\"}}"
create_lock_body5="{\"path\":\"file5\"}}"

date_regex="[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\+[0-9]{2}:[0-9]{2}"

sleep 5

# User1 create lock for path file1
run_with_header_capturing_outputs "curl -X POST 'https://$base_url' -v -H 'Authorization: Bearer $token_user1' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$create_lock_body1' --insecure"
expect_stdout_to_contain "{\"lock\":{\"id\":\"1\",\"path\":\"file1\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"user1\"}}}"
expect_stderr_to_contain "HTTP/1.1 201 Created"

# User2 can't create another one for the same path
run_with_header_capturing_outputs "curl -X POST 'https://$base_url' -v -H 'Authorization: Bearer $token_user2' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$create_lock_body1' --insecure"
expect_stdout_to_contain "{\"lock\":{\"id\":\"1\",\"path\":\"file1\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"user1\"}},\"message\":\"already created lock\"}"
expect_stderr_to_contain "HTTP/1.1 409 Conflict"

# User2 create lock for path file2
run_with_header_capturing_outputs "curl -X POST 'https://$base_url' -v -H 'Authorization: Bearer $token_user2' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$create_lock_body2' --insecure"
expect_stdout_to_contain "{\"lock\":{\"id\":\"2\",\"path\":\"file2\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"user2\"}}}"
expect_stderr_to_contain "HTTP/1.1 201 Created"

# List all locks
run_with_header_capturing_outputs "curl -X GET 'https://$base_url' -v -H 'Authorization: Bearer $token_user1' -s -H 'Content-Type: application/json' -H 'Accept: */*' --insecure"
expect_stdout_to_contain "{\"locks\":\[{\"id\":\"1\",\"path\":\"file1\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"user1\"}},{\"id\":\"2\",\"path\":\"file2\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"user2\"}}\]}"
expect_stderr_to_contain "HTTP/1.1 200 OK"

# List all locks for verification, from user1 perspective
run_with_header_capturing_outputs "curl -X POST 'https://$base_url/verify' -v -H 'Authorization: Bearer $token_user1' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '{}' --insecure"
expect_stdout_to_contain "{\"ours\":\[{\"id\":\"1\",\"path\":\"file1\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"user1\"}}\],\"theirs\":\[{\"id\":\"2\",\"path\":\"file2\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"user2\"}}\]}"
expect_stderr_to_contain "HTTP/1.1 200 OK"

# List all locks for verification, from user2 perspective
run_with_header_capturing_outputs "curl -X POST 'https://$base_url/verify' -v -H 'Authorization: Bearer $token_user2' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '{}' --insecure"
expect_stdout_to_contain "{\"ours\":\[{\"id\":\"2\",\"path\":\"file2\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"user2\"}}\],\"theirs\":\[{\"id\":\"1\",\"path\":\"file1\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"user1\"}}\]}"
expect_stderr_to_contain "HTTP/1.1 200 OK"

# Create a few more locks for pagination
run_with_header_capturing_outputs "curl -X POST 'https://$base_url' -v -H 'Authorization: Bearer $token_user1' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$create_lock_body3' --insecure"
run_with_header_capturing_outputs "curl -X POST 'https://$base_url' -v -H 'Authorization: Bearer $token_user1' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$create_lock_body4' --insecure"

# Pagination is global (not per user)
run_with_header_capturing_outputs "curl -X POST 'https://$base_url/verify' -v -H 'Authorization: Bearer $token_user1' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$limit3' --insecure"
expect_stdout_to_contain "{\"ours\":\[{\"id\":\"1\",\"path\":\"file1\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"user1\"}},{\"id\":\"3\",\"path\":\"file3\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"user1\"}}\],\"theirs\":\[{\"id\":\"2\",\"path\":\"file2\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"user2\"}}\],\"next_cursor\":\"4\"}"
expect_stderr_to_contain "HTTP/1.1 200 OK"

run_with_header_capturing_outputs "curl -X POST 'https://$base_url/verify' -v -H 'Authorization: Bearer $token_user1' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$limit3cursor4' --insecure"
expect_stdout_to_contain "{\"ours\":\[{\"id\":\"4\",\"path\":\"file4\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"user1\"}}\],\"theirs\":\[\]}"
expect_stderr_to_contain "HTTP/1.1 200 OK"

# User2 can't delete lock created by user1
run_with_header_capturing_outputs "curl -X POST 'https://$base_url/1/unlock' -v -H 'Authorization: Bearer $token_user2' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '{}' --insecure"
expect_stdout_to_contain "{\"message\":\"Forbidden\"}"
expect_stderr_to_contain "HTTP/1.1 403 Forbidden"

# User2 can force delete lock created by user1
run_with_header_capturing_outputs "curl -X POST 'https://$base_url/1/unlock' -v -H 'Authorization: Bearer $token_user2' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$force_body' --insecure"
expect_stdout_to_contain "{\"lock\":{\"id\":\"1\",\"path\":\"file1\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"user1\"}}}"
expect_stderr_to_contain "HTTP/1.1 200 OK"

# User2 can now recreate lock for path file1
run_with_header_capturing_outputs "curl -X POST 'https://$base_url' -v -H 'Authorization: Bearer $token_user2' -s -H 'Content-Type: application/json' -H 'Accept: */*' -d '$create_lock_body1' --insecure"
expect_stdout_to_contain "{\"lock\":{\"id\":\"5\",\"path\":\"file1\",\"locked_at\":\"$date_regex\",\"owner\":{\"name\":\"user2\"}}}"
expect_stderr_to_contain "HTTP/1.1 201 Created"
