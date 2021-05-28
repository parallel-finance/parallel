#!/bin/bash

# Set variables
MYPATH=$(dirname "$0")
ME=`basename "$0"`
FULL_RUN_PATH=$(readlink -f "$ME")
FULL_DIR_PATH=$(dirname ${FULL_RUN_PATH})

DAEMON_PATH="/etc/systemd/system"

CURR_USER=$(whoami)

# Going to work directory
cd ${MYPATH}

# Define help function
function ul-show-help {
    echo -e "Usage:"
    echo -e "   ./${ME}                        Run ${ME}"
    if [[ $OS != 'win' ]]; then
    echo -e "   ./${ME} --stop                 Kill Parallel Vanilla Node"
    echo -e "   ./${ME} --daemonize            Daemonize Parallel Vanilla Nodea As Service"
    echo -e "   ./${ME} --undaemonize          Remove Parallel Service"
    echo -e "   ./${ME} --status               Show Daemons Status"
    fi
    echo -e "\n"
}

function kill-process {
    PID_ULS=`pgrep -f /$1`
    if [[ -n ${PID_ULS} ]]; then
        sudo kill ${PID_ULS}
        echo "[$1] killed"
        sleep 1s
    else
        echo "[$1] not found"
    fi
}

function kill-all-processes {
    kill-process "parallel"
}

function undaemonize-service {
    sudo systemctl stop $1
    sudo systemctl disable $1
    if [ -f "/etc/systemd/system/multi-user.target.wants/$1.service" ]; then
        sudo rm "/etc/systemd/multi-user.target.wants/system/$1.service"
    fi
    if [ -f "/etc/systemd/system/$1.service" ]; then
        sudo rm "/etc/systemd/system/$1.service"
    fi
    sudo systemctl daemon-reload
    sudo systemctl reset-failed
    echo "[$1] undaemonized"
}

function undaemonize-all-service {
    undaemonize-service "parallel-vanilla-node"
}

function daemonize-service {
    # Deamonize
    echo ${DAEMON_PATH}
    sudo cp ./$SERVICE_FILE ${DAEMON_PATH}/
    sudo chmod 755 ${DAEMON_PATH}/$SERVICE_FILE
    sudo chown root:root ${DAEMON_PATH}/$SERVICE_FILE
    sudo systemctl link ${DAEMON_PATH}/$SERVICE_FILE

    echo -e "Run dameon"
    # Run deamon right now
    sudo systemctl enable $1
    sudo systemctl daemon-reload
    sudo systemctl start $1
    sudo systemctl reenable $1
}

function daemonize-all-service {
    SERVICE="parallel-vanilla-node"
    SERVICE_FILE="parallel-vanilla-node.service"
    if [ -f $SERVICE_FILE ]; then
        rm $SERVICE_FILE
    fi
    touch $"$SERVICE.service"
    printf "[Unit]\nDescription=Parallel Vanilla Node\nRequires=network.target\nAfter=multi-user.target graphical.target network.target syslog.target\nWants=network.target\n\n[Service]\nUser=${CURR_USER}\nRestart=always\nRestartSec=5\nWorkingDirectory=${FULL_DIR_PATH}\nExecStart=${FULL_DIR_PATH}/parallel -d ../parallel-chain --chain testnet --alice --rpc-cors all --rpc-methods=Unsafe --unsafe-rpc-external --unsafe-ws-external\n\n[Install]\nWantedBy=multi-user.target\n" > $SERVICE_FILE
    daemonize-service $SERVICE
}

# Catch parameters
if [ $1 == '--stop' ]; then

    kill-all-processes

elif [ $1 == '--daemonize' ]; then

    kill-all-processes

    undaemonize-all-service

    daemonize-all-service

elif [ $1 == '--undaemonize' ]; then

    kill-all-processes

    undaemonize-all-service

elif [ $1 == '--status' ]; then

    systemctl status  "parallel-vanilla-node"

else

    ul-show-help
fi
