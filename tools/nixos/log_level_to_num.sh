#!/bin/sh

# SPDX-License-Identifier: MPL-2.0

# Maps LOG_LEVEL names to kernel `loglevel=0..=8`.
# See Makefile and `ostd::early_cmdline_parser`.
level="${1:-error}"

case "${level}" in
    [0-8]) echo "${level}" ;;
    off) echo 0 ;;
    emerg) echo 1 ;;
    alert) echo 2 ;;
    crit) echo 3 ;;
    error) echo 4 ;;
    warn|warning) echo 5 ;;
    notice) echo 6 ;;
    info) echo 7 ;;
    debug) echo 8 ;;
    *) echo 4 ;;
esac
