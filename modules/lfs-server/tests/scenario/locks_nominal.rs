use crate::common::app_utils::ClientHelper;
use crate::{assert_match, assert_response_eq, assert_response_match};
use axum::http::{Method, StatusCode};

const UPLOAD_TOKEN_USER_1: &str = "Bearer eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiI1MDAwMDAwMDAwMDAwIiwib3BlcmF0aW9uIjoidXBsb2FkIiwicmVwbyI6InRlc3RpbmciLCJ1c2VyIjoidXNlcjEifQ.vGMY3IXRPcOvxu1Fbxen7b31L2jIUvv8msPU66vfL2c";
const DOWNLOAD_TOKEN_USER_1: &str = "Bearer eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiI1MDAwMDAwMDAwMDAwIiwib3BlcmF0aW9uIjoiZG93bmxvYWQiLCJyZXBvIjoidGVzdGluZyIsInVzZXIiOiJ1c2VyMSJ9.uO4n5bm3pE6gDuBFmMRo8pncrg6dnB6z_P3fqQ4PVzM";
const UPLOAD_TOKEN_USER_2: &str = "Bearer eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiI1MDAwMDAwMDAwMDAwIiwib3BlcmF0aW9uIjoidXBsb2FkIiwicmVwbyI6InRlc3RpbmciLCJ1c2VyIjoidXNlcjIifQ.I8Tg6mNdGvDd9KvyywQJsgcqzV38hk6xYhsCvfvrro8";
const DOWNLOAD_TOKEN_USER_2: &str = "Bearer eyJhbGciOiJIUzI1NiJ9.eyJleHAiOiI1MDAwMDAwMDAwMDAwIiwib3BlcmF0aW9uIjoiZG93bmxvYWQiLCJyZXBvIjoidGVzdGluZyIsInVzZXIiOiJ1c2VyMiJ9.bTSchEQBp6m3MpgF_B2Wfxq7JEXMLOvv0cMGcMAmYpQ";

pub async fn locks_nominal(mut app: ClientHelper) {
    // 1) User 1 creates a lock, but with the download token
    assert_response_eq!(
        app.lock("testing", "foo/bar.bin", "master", DOWNLOAD_TOKEN_USER_1),
        StatusCode::UNAUTHORIZED,
        "{\"message\":\"Unauthorized\"}"
    );

    // 2) User 1 creates a lock, but with the upload token now
    assert_response_match!(
        app.lock("testing", "foo/bar.bin", "master", UPLOAD_TOKEN_USER_1),
        StatusCode::CREATED,
        r#"\{"lock":\{"id":"1","path":"foo/bar.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user1"}}}"#
    );

    // 3) User 1 tries to create a lock on the same path, but it fails
    assert_response_match!(
        app.lock("testing", "foo/bar.bin", "master", UPLOAD_TOKEN_USER_1),
        StatusCode::CONFLICT,
        r#"\{"lock":\{"id":"1","path":"foo/bar.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user1"}},"message":"already created lock"}"#
    );

    // 4) User 2 tries to create a lock on the same path, but it fails
    assert_response_match!(
        app.lock("testing", "foo/bar.bin", "master", UPLOAD_TOKEN_USER_2),
        StatusCode::CONFLICT,
        r#"\{"lock":\{"id":"1","path":"foo/bar.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user1"}},"message":"already created lock"}"#
    );

    // 5) User 1 tries to create a lock on a different path, it works
    assert_response_match!(
        app.lock("testing", "foo/bar2.bin", "master", UPLOAD_TOKEN_USER_1),
        StatusCode::CREATED,
        r#"\{"lock":\{"id":"2","path":"foo/bar2.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user1"}}}"#
    );

    // 6) User 1 list the locks with a download token
    assert_response_match!(
        app.get_json("/locks?repo=testing", DOWNLOAD_TOKEN_USER_1),
        StatusCode::OK,
        r#"\{"locks":\[\{"id":"1","path":"foo/bar.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user1"}},\{"id":"2","path":"foo/bar2.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user1"}}]}"#
    );

    // 7) But it also works with upload token
    assert_response_match!(
        app.send_json(Method::GET, "/locks?repo=testing", UPLOAD_TOKEN_USER_1, ""),
        StatusCode::OK,
        r#"\{"locks":\[\{"id":"1","path":"foo/bar.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user1"}},\{"id":"2","path":"foo/bar2.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user1"}}]}"#
    );

    // 8) And user2 see the same
    assert_response_match!(
        app.get_json("/locks?repo=testing", DOWNLOAD_TOKEN_USER_2,),
        StatusCode::OK,
        r#"\{"locks":\[\{"id":"1","path":"foo/bar.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user1"}},\{"id":"2","path":"foo/bar2.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user1"}}]}"#
    );

    // 9) We can filter locks by path
    assert_response_match!(
        app.get_json(
            "/locks?repo=testing&path=foo/bar.bin",
            DOWNLOAD_TOKEN_USER_2,
        ),
        StatusCode::OK,
        r#"\{"locks":\[\{"id":"1","path":"foo/bar.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user1"}}]}"#
    );

    // 10) We can filter locks by id
    assert_response_match!(
        app.get_json("/locks?repo=testing&id=2", DOWNLOAD_TOKEN_USER_2,),
        StatusCode::OK,
        r#"\{"locks":\[\{"id":"2","path":"foo/bar2.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user1"}}]}"#
    );

    // 11) User 2 create a lock
    app.lock("testing", "foo/u2.bin", "master", UPLOAD_TOKEN_USER_2)
        .await;

    // 12) User 1 can list locks for verifications with a download token
    assert_response_match!(
        app.post_json("/locks/verify?repo=testing", DOWNLOAD_TOKEN_USER_1, "{}"),
        StatusCode::OK,
        r#"\{"ours":\[\{"id":"1","path":"foo/bar.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user1"}},\{"id":"2","path":"foo/bar2.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user1"}}],"theirs":\[\{"id":"3","path":"foo/u2.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user2"}}]}"#
    );

    // 13) User 2 can list locks for verifications with a download token, and get the reversed result
    assert_response_match!(
        app.post_json("/locks/verify?repo=testing", DOWNLOAD_TOKEN_USER_2, "{}"),
        StatusCode::OK,
        r#"\{"ours":\[\{"id":"3","path":"foo/u2.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user2"}}],"theirs":\[\{"id":"1","path":"foo/bar.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user1"}},\{"id":"2","path":"foo/bar2.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user1"}}]}"#
    );

    // 14) Create two more lock to test limits
    app.lock("testing", "foo/bar3.bin", "master", UPLOAD_TOKEN_USER_1)
        .await;
    app.lock("testing", "foo/bar4.bin", "master", UPLOAD_TOKEN_USER_1)
        .await;

    // 15) We can limit to 3 locks
    assert_response_match!(
        app.get_json("/locks?repo=testing&limit=3", DOWNLOAD_TOKEN_USER_2,),
        StatusCode::OK,
        r#"\{"locks":\[\{"id":"1","path":"foo/bar.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user1"}},\{"id":"2","path":"foo/bar2.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user1"}},\{"id":"3","path":"foo/u2.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user2"}}],"next_cursor":"4"}"#
    );

    // 16) We can limit down to 1 lock, starting from the 4th
    assert_response_match!(
        app.get_json(
            "/locks?repo=testing&limit=1&cursor=4",
            DOWNLOAD_TOKEN_USER_2,
        ),
        StatusCode::OK,
        r#"\{"locks":\[\{"id":"4","path":"foo/bar3.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user1"}}],"next_cursor":"5"}"#
    );

    // 17) Limit of 0 means no locks
    assert_response_eq!(
        app.get_json("/locks?repo=testing&limit=0", DOWNLOAD_TOKEN_USER_2,),
        StatusCode::OK,
        "{\"locks\":[],\"next_cursor\":\"2\"}"
    );

    // 18) NaN limit should fail
    assert_response_eq!(
        app.get_json("/locks?repo=testing&limit=foo", DOWNLOAD_TOKEN_USER_2,),
        StatusCode::UNPROCESSABLE_ENTITY,
        "{\"message\":\"InvalidLimit\"}"
    );

    // 19) When listing locks for verification, we can apply a limit that apply before the separation between ours and theirs,
    //     So a limit of 3 captures both 2 locks of ours and 1 lock of theirs
    assert_response_match!(
        app.post_json(
            "/locks/verify?repo=testing",
            DOWNLOAD_TOKEN_USER_1,
            "{\"limit\":\"3\"}"
        ),
        StatusCode::OK,
        r#"\{"ours":\[\{"id":"1","path":"foo/bar.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user1"}},\{"id":"2","path":"foo/bar2.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user1"}}],"theirs":\[\{"id":"3","path":"foo/u2.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user2"}}],"next_cursor":"4"}"#
    );

    // 20) But for user2, with a limit of 2, we only get 2 locks of theirs
    assert_response_match!(
        app.post_json(
            "/locks/verify?repo=testing",
            DOWNLOAD_TOKEN_USER_2,
            "{\"limit\":\"2\"}"
        ),
        StatusCode::OK,
        r#"\{"ours":\[\],"theirs":\[\{"id":"1","path":"foo/bar.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user1"}},\{"id":"2","path":"foo/bar2.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user1"}}],"next_cursor":"3"}"#
    );

    // 21) And then on next call, we get our lock and the next one of theirs
    assert_response_match!(
        app.post_json(
            "/locks/verify?repo=testing",
            DOWNLOAD_TOKEN_USER_2,
            "{\"limit\":\"2\",\"cursor\":\"3\"}"
        ),
        StatusCode::OK,
        r#"\{"ours":\[\{"id":"3","path":"foo/u2.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user2"}}],"theirs":\[\{"id":"4","path":"foo/bar3.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user1"}}],"next_cursor":"5"}"#
    );

    // 22) User1 unlock lock 1 with a download token, shall fail
    assert_response_eq!(
        app.post_json("/locks/1/unlock?repo=testing", DOWNLOAD_TOKEN_USER_1, "{}"),
        StatusCode::UNAUTHORIZED,
        "{\"message\":\"Unauthorized\"}"
    );

    // 23) User1 unlock lock 1 with an upload token, ok
    assert_response_match!(
        app.post_json("/locks/1/unlock?repo=testing", UPLOAD_TOKEN_USER_1, "{}"),
        StatusCode::OK,
        r#"\{"lock":\{"id":"1","path":"foo/bar.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user1"}}}"#
    );

    // 24) So now lock 1 do not appear anymore
    assert_response_eq!(
        app.get_json("/locks?repo=testing&id=1", UPLOAD_TOKEN_USER_1,),
        StatusCode::OK,
        r#"{"locks":[]}"#
    );

    // 25) User1 try to unlock lock 1 again, but it fails, as not found
    assert_response_eq!(
        app.post_json("/locks/1/unlock?repo=testing", UPLOAD_TOKEN_USER_1, "{}"),
        StatusCode::NOT_FOUND,
        "{\"message\":\"Not found\"}"
    );

    // 26) User1 try to unlock lock 3 of user 2, but it fails, as forbidden
    assert_response_eq!(
        app.post_json("/locks/3/unlock?repo=testing", UPLOAD_TOKEN_USER_1, "{}"),
        StatusCode::FORBIDDEN,
        "{\"message\":\"Forbidden\"}"
    );

    // 26) User1 force unlock lock 3 of user 2, succeed
    assert_response_match!(
        app.post_json(
            "/locks/3/unlock?repo=testing",
            UPLOAD_TOKEN_USER_1,
            "{\"force\":true}"
        ),
        StatusCode::OK,
        r#"\{"lock":\{"id":"3","path":"foo/u2.bin","locked_at":"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\+(\d{2}):(\d{2})","owner":\{"name":"user2"}}}"#
    );

    // 27) User1 unlock other locks, succeed
    for i in 2..=5 {
        app.post_json(
            format!("/locks/{}/unlock?repo=testing", i).as_str(),
            UPLOAD_TOKEN_USER_1,
            "{}",
        )
        .await;
    }

    // 28) User1 can list locks and get nothing
    assert_response_eq!(
        app.send_json(Method::GET, "/locks?repo=testing", UPLOAD_TOKEN_USER_1, ""),
        StatusCode::OK,
        "{\"locks\":[]}"
    );
}
