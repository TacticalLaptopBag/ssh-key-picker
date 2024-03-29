#!/bin/bash

if [ -z "${INSTALL_DIR}" ]; then
    if [ "$UID" -eq 0 ]; then
        INSTALL_DIR="/usr/bin"
    else
        INSTALL_DIR="$HOME/.local/bin"
    fi
    echo "INSTALL_DIR not set. Defaulting to ${INSTALL_DIR}"
fi

install -m 557 ssh-key-picker.lua ${INSTALL_DIR}/ssh-key-picker

echo "Installed ssh-key-picker."
