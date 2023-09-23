echo "test" > /tmp/test.txt


init() {
    sleep 5
    mc alias set myminio http://localhost:9000 minio_access_key minio_secret_key
    mc mb myminio/bucket
    mc cp /tmp/test.txt myminio/bucket/repo/objects/test.txt
}

init & minio server /data --console-address ":9001"
