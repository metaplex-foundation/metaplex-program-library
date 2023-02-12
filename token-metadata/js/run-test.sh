#!/bin/bash
#
# To run this script, you need:
#  - npm install -g esbuild-runner 
#  - npm install -g tap-spec

# error output colour
RED() { echo $'\e[1;31m'$1$'\e[0m'; }
RUN_ALL=0

# check whether we are running all test files or not

while getopts a-: optchar; do
    case "${optchar}" in
        a)
            RUN_ALL=1 ;;
        -) 
            case "${OPTARG}" in
                all) RUN_ALL=1 ;;
                *) ;;
            esac ;;
        *) ;;
    esac
done

# runs single or multiple tests

if [ $RUN_ALL -eq 1 ]; then
    for file in `ls test/*.test.ts`
    do
        esr $file | tap-spec
    done
else
    if [ ! -z "$1" ] && [[ -f "$1" ]]; then
        esr $1 | tap-spec
    else
        echo "$(RED "Error: ")Please specify a test file or [-a | --all] to run all tests"
        exit 1
    fi
fi
