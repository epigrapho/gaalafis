#!/bin/bash

bold=$(tput bold)
normal=$(tput sgr0)

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

# get the architecture from json
architecture=$(jq -r "to_entries[] | select(.value | contains([$1])) | .key" ./architectures.json)
exit_code=0
for architecture in $architecture; do
    # title
    echo ""
    echo ""
    echo "${bold}Running test case $1 on architecture $architecture ${normal}"

    # start timer
    start=$(date +%s)

    # run the test case
    run_and_print "docker-compose -f architectures/$architecture.docker-compose.yaml run tester_client bash /root/run_test.sh $1" "[up]"
    status=$?
    end_test=$(date +%s)

    # fetch log files if status is not 0
    if [ $status -ne 0 ]; then
        get_logs "architectures_gitolite_1" "/var/lib/git/log/output.log"
        run_and_print "docker-compose -f architectures/$architecture.docker-compose.yaml logs" "[logs]"
    fi


    # cleanup
    run_and_print "docker-compose -f architectures/$architecture.docker-compose.yaml down -v -t 0" "[down]"
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
        exit_code="$status"
    fi

    # end timer and print the duration
    end=$(date +%s)
    echo "    > test took $(($end_test-$start)) seconds"
    echo "    > cleanup took $(($end-$end_test)) seconds"
done


exit $exit_code

