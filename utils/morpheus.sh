#!/bin/bash

function usage() {
    echo  "Usage:   $0 -a MORPHEUS_ADDRESS [-p PRE_DELAY] [-w] COMMAND

Parameters:
  -a MORPHEUS_ADDRESS: address/name of the docker container running morpheus
  -p PRE_DELAY: Sets the PRE-SLEEP delay, in seconds (default 60)
  -w: Wait for BalenaCloud connection to check update before sending request

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

function check_vpn_connected() {
    curl "$BALENA_SUPERVISOR_ADDRESS/v2/device/vpn?apikey=$BALENA_SUPERVISOR_API_KEY" \
    | jq ".vpn | .connected"
}

function wait_vpn_connecter() {
    echo "### Waiting for Cloud connection to be established"
    while : ; do
        VPN_CONNECTED=$(check_vpn_connected)
        if [ ! -z ${VPN_CONNECTED} ]; then
            if [ ${VPN_CONNECTED} = "true" ]; then
                return 0
            fi
        fi
        sleep 5
    done
}

function wait_for_update() {
    echo "### Checking for pending update"
    CHECK=$(check_update)
    while [ -z "${CHECK}" ] || [ "true" = "${CHECK}" ]
    do
        sleep 5
        CHECK=$(check_update)
    done
}

WAIT_VPN=false

while getopts "a:p:hw" o; do
    case $o in
        a)
            MORPHEUS_ADDRESS=${OPTARG}
            ;;
        
        p)
            PRE_DELAY_PARAM=${OPTARG}
            ;;
        
        w)
            WAIT_VPN=true
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
    RESULT=$(curl -X GET "http://${MORPHEUS_ADDRESS}:5555/$1" | jq "$2")

    if [ -z ${RESULT} ]; then 
        echo "ERROR: Failed during request to http://${MORPHEUS_ADDRESS}:5555/$1" 1>&2
        echo "failed"
    fi
}

if ${WAIT_VPN};then
    wait_vpn_connecter
    wait_for_update
fi

case ${COMMAND} in 
    TimeSleep)
        DURATION=${@:$OPTIND+1:1}
        if [ -z DURATION ]; then
            echo "Error: missing sleep duration" 1>&2
            usage
        fi
        echo "### Requesting Time Sleep"
        # Request Time sleep
        RESULT=$(request "sleep_time/${PRE_DELAY}/${DURATION}" ".SleepTime | .feedback")
        if [ "$RESULT" = "1" ]; then
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
        echo "### Requesting GPIO waked Sleep"
        RESULT=$(request "sleep_pin/${PRE_DELAY}/${ACTIVE_STATE}" ".SleepPin | .success")
        if [ "$RESULT" = "1" ]; then
            exit 0
        else
            exit 1
        fi
        ;;

    *)
        echo "ERROR: Unknown command" 1>&2
        usage
esac
