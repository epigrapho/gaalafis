run_with_format() {
    eval "$@"
    s=$?
    return $s
}

space () {
    echo "[space]"
}

header () {
    space
    echo "[cmd]$@"
}

ok () {
    space
    echo "[ok] $@"
}

run_with_header() {
    header "$@"
    run_with_format "$@"
    s=$?
    if [ $s -ne 0 ]; then
        exit $s
    fi
}

run_with_3_retries() {
    for i in {1..3}; do
        header "$@, try $i/3"
        run_with_format "$@"
        s=$?
        if [ $s -eq 0 ]; then
            return 0
        fi
        echo "    > retrying in 3 seconds..."
        sleep 3
    done
    if [ $s -ne 0 ]; then
        exit $s
    fi
}

expect_file_to_contains() {
    if [ ! -f "$1" ]; then
        header "Expect file $1 to contains '$2'"
        echo "    > FAIL: file $1 not found"
        exit 1
    fi
    if ! grep -q "$2" "$1"; then
        header "Expect file $1 to contains '$2'"
        echo "    > FAIL: file $1 does not contains $2"
        exit 1
    fi
    ok "Expect file $1 to contains '$2'"
}

expect_folder_to_exists() {
    if [ ! -d "$1" ]; then
        header "Expect folder $1 to exists"
        echo "    > FAIL: $1 folder not found"
        exit 1
    fi
    ok "Expect folder $1 to exists"
}
