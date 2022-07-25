#!/bin/bash

function usage() {
    echo  "Usage:   $0 -a MORPHEUS_ADDRESS [-p PRE_DELAY] COMMAND

Parameters:
  -a MORPHEUS_ADDRESS: address/name of the docker container running morpheus
  -p PRE_DELAY: Sets the PRE-SLEEP delay, in seconds (default 60)

where COMMAND can be:

  - TimeSleep DELAY: to trigger a DELAY seconds sleep period
  - InputSleep ACTIVE_STATE: to trigger an infinite duration sleep with GPIO wakeup (wake up on ACTIVE_STATE)" 1>&2

    exit 1
}

# Wait for the supervisor to be ready
function check_update() {
   export UPDATE=$(curl -sX GET "$BALENA_SUPERVISOR_ADDRESS/v1/device?apikey=$BALENA_SUPERVISOR_API_KEY" \
   -H "Content-Type: application/json" | \
   jq ".update_pending")
   echo ${UPDATE}
}

function wait_for_update() {
    echo "### Checking for pending update"
    CHECK=$(check_update)
    while [ [ -z ${CHECK} ] || [ "true" = "${CHECK}" ] ]
    do
        sleep 5
        CHECK=$(check_update)
    done
}

while getopts "a:p:h" o; do
    case $o in
        a)
            MORPHEUS_ADDRESS=${OPTARG}
            ;;
        
        p)
            PRE_DELAY_PARAM=${OPTARG}
            ;;
        
        *)
            usage
            ;;

    esac
done

# -a is a MANDATORY PARAMETER
if [ -z ${MORPHEUS_ADDRESS} ]; then
    echo "Error: Missing address" 1>&2
    usage
fi

PRE_DELAY=${PRE_DELAY_PARAM:-60}

# Grab the command
COMMAND=${@:$OPTIND:1}

if [ -z ${COMMAND} ]; then
   echo "Error: Missing COMMAND" 1>&2
   usage
fi


request() {
    RESULT=$(curl -X GET "http://${MORPHEUS_ADDRESS}:5555/$1")
    echo "READ: ${RESULT}" 1>&2
    RESULT=$(echo "${RESULT}" | jq $2)

    if [ -z ${RESULT} ]; then 
        echo "ERROR: Failed during request to http://${MORPHEUS_ADDRESS}:5555/$1" 1>&2
        echo "failed"
    fi
}

case ${COMMAND} in 
    TimeSleep)
        DURATION=${@:$OPTIND+1:1}
        if [ -z DURATION ]; then
            echo "Error: missing sleep duration" 1>&2
            usage
        fi
        # Request Time sleep
        RESULT=$(request "sleep_time/${PRE_DELAY}/${DURATION}" ".SleepTime | .feedback")
        if [ "$RESULT" = "true" ]; then
            exit 0
        else
            exit 1
        fi
        ;;

    InputSleep)
        ACTIVE_STATE=${@:$OPTIND+1:1}
        if [ -z ACTIVE_STATE ]; then
            echo "Error: missing active state (1 or 0)" 1>&2
            usage
        fi
        # Request input sleep
        RESULT=$(request "sleep_pin/${PRE_DELAY}/${ACTIVE_STATE}" ".SleepPin | .success")
        if [ "$RESULT" = "true" ]; then
            exit 0
        else
            exit 1
        fi
        ;;

    *)
        echo "ERROR: Unknown command" 1>&2
        usage
esac
