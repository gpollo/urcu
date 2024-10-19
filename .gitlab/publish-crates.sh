#!/bin/bash

#################
# cargo helpers #
#################

function cargo() {
    >&2 echo " "
    >&2 echo '$' cargo "$@"
    >&2 echo " "

    command cargo "$@"
}

function cargo_login() {
    cargo login "${CRATES_IO_TOKEN}"
}

#################
# crate helpers #
#################

function crate_name() {
    local dir="$1"
    local crate_name
    local crate_version

    crate_name=$(tq -f "${dir}/Cargo.toml" "package.name")
    crate_version=$(tq -f "${dir}/Cargo.toml" "package.version")

    echo -n "${crate_name}@${crate_version}"
}

function crate_released() {
    local dir="$1"

    cargo -Z unstable-options -C /tmp info "$(crate_name "${dir}")" > /dev/null
}

function crate_publish() {
    local dir="$1"

    if ! crate_released "${dir}"; then
        pushd "${dir}"
        cargo publish "${@:2}"
        popd
    fi
}

##############
# entrypoint #
##############

set -e
cd "$(dirname "$0")" || exit 1

cargo_login
crate_publish ../urcu-src
crate_publish ../urcu-common-sys
crate_publish ../urcu-sys
crate_publish ../urcu-cds-sys
crate_publish ../urcu-bp-sys
crate_publish ../urcu-mb-sys
crate_publish ../urcu-memb-sys
crate_publish ../urcu-qsbr-sys
cp ../README.md ../urcu/README.md
sed -i 's#../../README.md#../README.md#g' ../urcu/src/lib.rs
crate_publish ../urcu --allow-dirty