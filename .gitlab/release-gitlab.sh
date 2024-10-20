#!/bin/bash

###############
# git helpers #
###############

function git_current_hash() {
    git rev-parse --short HEAD
}

#################
# crate helpers #
#################

function crate_name() {
    local dir="$1"

    tq -f "${dir}/Cargo.toml" "package.name"
}

function crate_version() {
    local dir="$1"

    tq -f "${dir}/Cargo.toml" "package.version"
}

##################
# gitlab helpers #
##################

function gitlab_release_version() {
    echo -n "v$(crate_version urcu)"
}

function gitlab_release_name() {
    echo -n "Userspace RCU $(gitlab_release_version)"
}

function gitlab_release_description() {
    echo "This release contains the following crates."
    echo ""
    echo "| Crate                           | Version                            |"
    echo "|:--------------------------------|:-----------------------------------|"
    echo "| \`$(crate_name urcu-bp-sys)\`   | \`$(crate_version urcu-bp-sys)\`   |"
    echo "| \`$(crate_name urcu-cds-sys)\`  | \`$(crate_version urcu-cds-sys)\`  |"
    echo "| \`$(crate_name urcu-mb-sys)\`   | \`$(crate_version urcu-mb-sys)\`   |"
    echo "| \`$(crate_name urcu-memb-sys)\` | \`$(crate_version urcu-memb-sys)\` |"
    echo "| \`$(crate_name urcu-qsbr-sys)\` | \`$(crate_version urcu-qsbr-sys)\` |"
    echo "| \`$(crate_name urcu-src)\`      | \`$(crate_version urcu-src)\`      |"
    echo "| \`$(crate_name urcu-sys)\`      | \`$(crate_version urcu-sys)\`      |"
    echo "| \`$(crate_name urcu)\`          | \`$(crate_version urcu)\`          |"
}

function gitlab_release() {
    glab config set token "${GITLAB_RELEASE_TOKEN}"
    glab release create "$(gitlab_release_version)" \
        --name "$(gitlab_release_name)" \
        --notes "$(gitlab_release_description)" \
        --ref "$(git_current_hash)"
}

##############
# entrypoint #
##############

set -e
cd "$(dirname "$0")/.." || exit 1

gitlab_release