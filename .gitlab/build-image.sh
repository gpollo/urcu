#!/usr/bin/bash

###############
# git helpers #
###############

function git_list_changed_files() {
    echo git diff --name-status "${CI_MERGE_REQUEST_TARGET_BRANCH_SHA}..${CI_COMMIT_SHORT_SHA}"
}

#################
# crane helpers #
#################

function crane() {
    >&2 echo " "
    >&2 echo '$' crane "$@"
    >&2 echo " "

    /tmp/crane "$@"
}

function crane_install() {
    local version=v0.20.2
    local os="Linux"
    local arch="x86_64"
    local url_base="https://github.com/google/go-containerregistry/releases/download"
    local url_file="${version}/go-containerregistry_${os}_${arch}.tar.gz"
    local url="${url_base}/${url_file}"

    curl -sL "${url}" > /tmp/go-containerregistry.tar.gz
    tar -zxvf /tmp/go-containerregistry.tar.gz -C /tmp/ crane
}

function crane_login() {
    crane auth login -u "${CI_REGISTRY_USER}" -p "${CI_REGISTRY_PASSWORD}" "${CI_REGISTRY}"
}

function crane_image_exist() {
    local image_name="$1"

    if crane manifest "${image_name}"; then
        return 0
    else
        return 1
    fi
}

##################
# podman helpers #
##################

function podman() {
    >&2 echo " "
    >&2 echo '$' podman "$@"
    >&2 echo " "

    command podman "$@"
}

function podman_login() {
    podman login -u "${CI_REGISTRY_USER}" -p "${CI_REGISTRY_PASSWORD}" "${CI_REGISTRY}"
}

#################
# image helpers #
#################

function image_name() {
    local image="$1"
    local image_version
    local image_name

    image_version="$(cat "image-${image}/VERSION")"
    image_name="${CI_REGISTRY_IMAGE}/${image}:${image_version}"

    echo -n "${image_name}"
}

function image_name_git() {
    local image="$1"

    echo -n "$(image_name "${image}")-${CI_COMMIT_SHORT_SHA}"
}

function image_needs_rebuild() {
    local image="$1"

    if crane_image_exist "$(image_name "${image}")"; then
        return 1
    fi

    if [[ "${CI_MERGE_REQUEST_ID}" == "" ]]; then
        if [[ "${CI_COMMIT_BRANCH}" == "master" ]]; then
            return 0
        fi
    fi

    if git_list_changed_files | grep "image-${image}/VERSION"; then
        return 0
    fi

    return 0
}

function image_build_args() {
    local base_image="$1"

    if [[ "${base_image}" != "" ]]; then
        echo -n --build-arg BASE_IMAGE="$(image_name_git "${base_image}")"
    else
        echo -n
    fi
}

function image_setup() {
    local target_image="$1"
    local base_image="$2"

    if crane_image_exist "$(image_name_git "${target_image}")"; then
        return
    fi

    if image_needs_rebuild "${target_image}"; then
        # shellcheck disable=SC2046
        podman build "image-${target_image}" \
            --tag "$(image_name "${target_image}")" \
            $(image_build_args "${base_image}")
    else
        podman pull "$(image_name "${target_image}")"
    fi

    podman tag "$(image_name "${target_image}")" "$(image_name_git "${target_image}")"
    podman push "$(image_name_git "${target_image}")"

    if [[ "${CI_COMMIT_BRANCH}" == "master" ]]; then
        podman push "$(image_name "${target_image}")"
    fi
}

##############
# entrypoint #
##############

function script_setup() {
    crane_install
    crane_login
    podman_login
}

set -e
cd "$(dirname "$0")" || exit 1

if [[ "$1" == "rust" ]]; then
    script_setup
    image_setup "rust"
elif [[ "$1" == "urcu-shared" ]]; then
    script_setup
    image_setup "urcu-shared" "rust"
elif [[ "$1" == "urcu-static" ]]; then
    script_setup
    image_setup "urcu-static" "rust"
fi