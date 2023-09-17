#!/bin/bash

# runner specific logs
log_as_runner() {
    echo "[runner] $1"
}

run_and_print() {
    echo ""
    bash $@ 2>&1 | sed 's/^/[test]/'
    s="${PIPESTATUS[0]}"
    return $s
}

# get a number as argument
if [ $# -eq 0 ]
then
    log_as_runner "No arguments supplied"
    exit 1
fi

# Find the first file with a matching numeric prefix
search_dir="tests"
numeric_prefix=$1
found_file=$(find "$search_dir" -type f -regex ".*/0*${numeric_prefix}[^0-9].*\.sh")

if [ -n "$found_file" ]; then
  log_as_runner "Found file: $found_file"
else
  log_as_runner "No file found with numeric prefix $numeric_prefix"
  exit 1  # Failure (non-zero exit code)
fi

chmod +x "$found_file"  # Make the script executable if it's not
run_and_print "$found_file"           # Run the .sh file
exit_code=$?
echo "[test][space]"
if [ $exit_code -eq 0 ]; then
    log_as_runner "✅ test case $found_file: PASS"
    exit 0  # Success (0 exit code)
else
    log_as_runner "❌ test case $found_file: FAIL"
    exit $exit_code  # Failure (non-zero exit code)
fi
