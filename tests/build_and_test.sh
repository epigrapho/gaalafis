#!/bin/bash

./build.sh

# Set the folder path
folder_path="./tests"

# Change directory to the specified folder
cd "$folder_path" || exit 1

numbers=()

# Use a for loop to iterate through files
for file in *; do
  # Check if the file name starts with a number
  if [[ "$file" =~ ^[0-9]+ ]]; then
    # Extract the number and print it
    number="${file%%_*}"  # Extracts the number before the first underscore
    numbers+=("$(echo "$number" | sed 's/^0*//')")
  fi
done

sorted_numbers=($(printf "%s\n" "${numbers[@]}" | sort -n))

# run the tests
results=()
cd ".." || exit 1
for num in "${sorted_numbers[@]}"; do
  ./start.sh "$num"
  results+=("$?")
done

# print the results
echo ""
echo "# ---------------------------------------------------------------------------- #"
echo "#                                    Results                                   #"
echo "# ---------------------------------------------------------------------------- #"
echo ""
for i in "${!sorted_numbers[@]}"; do
    if [[ "${results[i]}" -eq 0 ]]; then
        echo -e "    \e[32m✓ test case ${sorted_numbers[i]}\e[0m"
    else
        echo -e "    \e[31m✗ test case ${sorted_numbers[i]}\e[0m"
    fi
done
echo ""
echo "# ---------------------------------------------------------------------------- #"
