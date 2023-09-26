. ./tests/helpers.sh --source-only

sleep 3

# Clone the repo
run_with_header "git config --global user.email \"you@example.com\" \
    && git config --global user.name \"Your Name\" \
    && git config --global init.defaultBranch master \
    && git config --global http.sslverify false \
    && git clone git@gitolite:testing"
run_with_header "cd testing \
    && git lfs track '*.bin' \
    && git add .gitattributes"

# Genereate 100MB of files
header "Genereate 100 files of 1MB"
for i in {1..100}; do
    dd "if=/dev/urandom" "of=./test$i.bin" bs=4096 count=256 1>/dev/null 2>/dev/null
    git add test$i.bin
done
ok "Done"
run_with_header "git commit -m \"add files\" -q"

# Push over lfs, measure time
start=$(date +%s)
run_with_header "git push"
end=$(date +%s)
test_took_seconds=$end-$start

# Requirement is to push in less than 60 seconds 
if [ $test_took_seconds -gt 60 ]; then
    echo "    > FAIL: push took more than 60 seconds"
    exit 1
else
    ok "clone took $(($test_took_seconds)) seconds"
fi

# Clone from lfs, measure time
run_with_header "cd .."
start=$(date +%s)
run_with_header "git clone git@gitolite:testing testing2"
end=$(date +%s)
test_took_seconds=$end-$start

# Requirement is to clone in less than 60 seconds 
if [ $test_took_seconds -gt 60 ]; then
    echo "    > FAIL: clone took more than 60 seconds"
    exit 1
else
    ok "clone took $(($test_took_seconds)) seconds"
fi

# Verify we get the files correctly
run_with_header "cd testing2"
run_with_header "ls | grep -q \"test1.bin\""
run_with_header "ls | grep -q \"test27.bin\""
run_with_header "ls | grep -q \"test83.bin\""






