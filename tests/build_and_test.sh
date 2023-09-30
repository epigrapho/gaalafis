#!/bin/bash

bold=$(tput bold)
normal=$(tput sgr0)
pad_right() {
    echo $1 | sed -e :a -e "s/^.\{1,$2\}$/& /;ta"
}

start_build=$(date +%s)
./build.sh
end_build=$(date +%s)

# Set the folder path
folder_path="./tests"

# Change directory to the specified folder
cd "$folder_path" || exit 1

numbers=()
files=()

# Use a for loop to iterate through files
for file in *; do
  # Check if the file name starts with a number
  if [[ "$file" =~ ^[0-9]+ ]]; then
    # Extract the number and print it
    number="${file%%_*}"  # Extracts the number before the first underscore
    numbers+=("$(echo "$number" | sed 's/^0*//')")
    files+=("$file")
  fi
done

sorted_numbers=($(printf "%s\n" "${numbers[@]}" | sort -n))
end_scan=$(date +%s)

# run the tests
results=()
started_at=()
ended_at=()
cd ".." || exit 1
for num in "${sorted_numbers[@]}"; do
  started_at+=("$(date +%s)")
  ./start.sh "$num"
  results+=("$?")
  ended_at+=("$(date +%s)")
done
end_tests=$(date +%s)

# print the results
echo ""
echo ""
echo "┌──────────────────────────────────────────────────────────────────────────────┐"
echo "│                                    ${bold}Results${normal}                                   │"
echo "├──────────────────────────────────────────────────────────────────────────────┤"
echo "│                                                                              │"
echo -e "│    $(pad_right "Building images took $((${end_build}-${start_build}))s" 45)                            │ "
echo -e "│    $(pad_right "Scanning tests took $((${end_scan}-${end_build}))s" 45)                            │ "
echo "│                                                                              │"
echo "├──────────────────────────────────────────────────────────────────────────────┤"
echo "│                                                                              │"
for i in "${!sorted_numbers[@]}"; do
    duration=$((${ended_at[i]}-${started_at[i]}))
    file="${files[i]}"
    if [[ "${results[i]}" -eq 0 ]]; then
        echo -e "│    \e[32m✓ $(pad_right "test case ${file}" 56)\e[0m [ $(pad_right "$duration s" 8) ] │"
    else
        echo -e "│    \e[31m✗ $(pad_right "test case ${file}" 56)\e[0m [ $(pad_right "$duration s" 8) ] │"
    fi
done
echo "│                                                                              │"
echo "├──────────────────────────────────────────────────────────────────────────────┤"
echo "│                                                                              │"
echo -e "│    $(pad_right "Tests took $((${end_tests}-${start_build}))s" 45)                            │ "
echo "│                                                                              │"
echo "└──────────────────────────────────────────────────────────────────────────────┘"
