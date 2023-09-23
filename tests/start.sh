#!/bin/bash

bold=$(tput bold)
normal=$(tput sgr0)

# make it bold
echo "${bold}Running test case $1${normal}"

# get some stream in stdin, search for a prefix, and print the line with some formating 
format_line() {
    while read line; do
        if [[ $line == \[* ]]; then
            if [[ $line ==  \[*test\]*\[*cmd\]* ]]; then
                echo -e "    \e[34m|\e[0m   \e[33m⚙ [cmd] \e[0m\e[1m\e[34m${line:11}\e[0m"
            elif [[ $line ==  \[*test\]*\[*ok\]* ]]; then
                echo -e "    \e[34m|\e[0m   \e[32m✓ [ok]  \e[0m\e[1m\e[34m${line:11}\e[0m"
            elif [[ $line ==  \[*test\]*\[*space\]* ]]; then
                echo -e "    \e[34m|       \e[33m \e[0m       ${line:13}\e[0m"
            elif [[ $line == \[*test\]* ]]; then
                echo -e "    \e[34m|       \e[33m|\e[0m       ${line:6}\e[0m"
            elif [[ $line == \[*runner\]* ]]; then
                echo -e "    \e[37m> ${line:9}\e[0m"
            else
                echo -e "    \e[37m$ $line\e[0m"
            fi
        else
            echo -e "    \e[37m$ $line\e[0m"
        fi
    done
}

# run something, capture the stdout and stderr, print it with a prefix, and return the exit code
run_and_print() {
    echo ""
    echo "${bold}  $2${normal}"
    $1 2>&1 | format_line
    s="${PIPESTATUS[0]}"
    return $s
}

get_logs() {
    run_and_print "docker exec $1 cat $2" "[logs] try to get logs from $1:$2..."
    get_logs_status=$?
    if [[ $get_logs_status != 0 ]]; then 
        echo -e "    \e[37m$ No log found (exit code is $get_logs_status)\e[0m"
    fi
}

# start timer
start=$(date +%s)

# run the test case
run_and_print "docker-compose -f architectures/default.docker-compose.yaml run tester_client bash /root/run_test.sh $1" "[up]"
status=$?
end_test=$(date +%s)

# fetch log files
get_logs "architectures_gitolite_1" "/var/lib/git/log/output.log"

# cleanup
run_and_print "docker-compose -f architectures/default.docker-compose.yaml down -v -t 0" "[down]"
end_down=$(date +%s)

# print the result of the test case
if [ $status -eq 0 ]; then
    echo ""
    echo "${bold}✅ test case $1: ${normal}"
    echo "    > PASS"
else
    echo ""
    echo "${bold}❌ test case $1: ${normal}"
    echo "    > FAIL with status $status"
fi

# end timer and print the duration
end=$(date +%s)
echo "    > test took in $(($end_test-$start)) seconds"
echo "    > cleanup took $(($end-$end_test)) seconds"

exit $status

