#!/bin/sh
set -e

RESET="\\033[0m"
RED="\\033[31;1m"
GREEN="\\033[32;1m"
YELLOW="\\033[33;1m"
BLUE="\\033[34;1m"
WHITE="\\033[37;1m"

print_unsupported_platform()
{
    >&2 say_red "error: We're sorry, but it looks like Pulumi is not supported on your platform"
    >&2 say_red "       We support 64-bit versions of Linux and macOS and are interested in supporting"
    >&2 say_red "       more platforms.  Please open an issue at https://github.com/pulumi/pulumi and"
    >&2 say_red "       let us know what platform you're using!"
}

say_green()
{
    [ -z "${SILENT}" ] && printf "%b%s%b\\n" "${GREEN}" "$1" "${RESET}"
    return 0
}

say_red()
{
    printf "%b%s%b\\n" "${RED}" "$1" "${RESET}"
}

say_yellow()
{
    [ -z "${SILENT}" ] && printf "%b%s%b\\n" "${YELLOW}" "$1" "${RESET}"
    return 0
}

say_blue()
{
    [ -z "${SILENT}" ] && printf "%b%s%b\\n" "${BLUE}" "$1" "${RESET}"
    return 0
}

say_white()
{
    [ -z "${SILENT}" ] && printf "%b%s%b\\n" "${WHITE}" "$1" "${RESET}"
    return 0
}

at_exit()
{
    # shellcheck disable=SC2181
    # https://github.com/koalaman/shellcheck/wiki/SC2181
    # Disable because we don't actually know the command we're running
    STATUS="$?"
    if [ "$STATUS" -ne 0 ] && [ "$STATUS" -ne 33 ]; then
        >&2 say_red
        >&2 say_red "We're sorry, but it looks like something might have gone wrong during installation."
        >&2 say_red "If you need help, please join us on https://slack.pulumi.com/"
    fi
}

trap at_exit EXIT

VERSION=""
INSTALL_ROOT=""
NO_EDIT_PATH=""
SILENT=""
while [ $# -gt 0 ]; do
    case "$1" in
        --version)
            if [ "$2" != "latest" ]; then
                VERSION=$2
            fi
            ;;
        --silent)
            SILENT="--silent"
            ;;
        --install-root)
            INSTALL_ROOT=$2
            ;;
        --no-edit-path)
            NO_EDIT_PATH="true"
            ;;
     esac
     shift
done

if [ "${VERSION}" = "dev" ]; then
    IS_DEV_VERSION=true
    if ! VERSION=$(curl --retry 3 --fail --silent -L "https://www.pulumi.com/latest-dev-version"); then
        >&2 say_red "error: could not determine latest dev version of Pulumi, try passing --version X.Y.Z to"
        >&2 say_red "       install an explicit version, or no argument to get the latest release version"
        exit 1
    fi
fi

if [ -z "${VERSION}" ]; then

    # Query pulumi.com/latest-version for the most recent release. Because this approach
    # is now used by third parties as well (e.g., GitHub Actions virtual environments),
    # changes to this API should be made with care to avoid breaking any services that
    # rely on it (and ideally be accompanied by PRs to update them accordingly). Known
    # consumers of this API include:
    #
    # * https://github.com/actions/virtual-environments
    #

    if ! VERSION=$(curl --retry 3 --fail --silent -L "https://www.pulumi.com/latest-version"); then
        >&2 say_red "error: could not determine latest version of Pulumi, try passing --version X.Y.Z to"
        >&2 say_red "       install an explicit version"
        exit 1
    fi
fi

OS=""
case $(uname) in
    "Linux") OS="linux";;
    "Darwin") OS="darwin";;
    *)
        print_unsupported_platform
        exit 1
        ;;
esac

ARCH=""
case $(uname -m) in
    "x86_64") ARCH="x64";;
    "arm64") ARCH="arm64";;
    "aarch64") ARCH="arm64";;
    *)
        print_unsupported_platform
        exit 1
        ;;
esac

TARBALL_URL="https://github.com/pulumi/pulumi/releases/download/v${VERSION}/"
TARBALL_URL_FALLBACK="https://get.pulumi.com/releases/sdk/"
TARBALL_PATH=pulumi-v${VERSION}-${OS}-${ARCH}.tar.gz
PULUMI_INSTALL_ROOT=${INSTALL_ROOT}

if [ "$PULUMI_INSTALL_ROOT" = "" ]; then
    # Default to ~/.pulumi
    PULUMI_INSTALL_ROOT="${HOME}/.pulumi"
fi
PULUMI_CLI="${PULUMI_INSTALL_ROOT}/bin/pulumi"

if [ -d "${PULUMI_CLI}" ]; then
    say_red "error: ${PULUMI_CLI} already exists and is a directory, refusing to proceed."
    exit 1
elif [ ! -f "${PULUMI_CLI}" ]; then
    say_blue "=== Installing Pulumi ${VERSION} ==="
else
    say_blue "=== Upgrading Pulumi $(${PULUMI_CLI} version) to ${VERSION} ==="
fi

TARBALL_DEST=$(mktemp -t pulumi.tar.gz.XXXXXXXXXX)

PR_NUMBER=""
download_github_pr() {
    PR_NUMBER="$1"

    HEAD_SHA=$(
        curl --retry 2 --fail --silent -L \
            -H "Accept: application/vnd.github+json" \
            -H "X-GitHub-Api-Version: 2022-11-28" \
            "https://api.github.com/repos/pulumi/pulumi/pulls/${PR_NUMBER}" \
        | jq -r '.head.sha')

    if [ -z "${HEAD_SHA}" ]; then
        >&2 say_red "error: could not find HEAD SHA for PR ${PR_NUMBER}"
        exit 1
    fi
    >&2 say_white "+ Found PR ${PR_NUMBER} with HEAD SHA ${HEAD_SHA}"

    WORKFLOW_RUN_ID=$(
        curl --retry 2 --fail --silent -L \
            -H "Accept: application/vnd.github+json" \
            -H "X-GitHub-Api-Version: 2022-11-28" \
            "https://api.github.com/repos/pulumi/pulumi/actions/runs?head_sha=${HEAD_SHA}" \
        | jq -r '
            .workflow_runs
            | map(select(.referenced_workflows[].path
                            | contains("ci.yml")))
            | max_by(.run_started_at) | .id')

    if [ -z "${WORKFLOW_RUN_ID}" ]; then
        >&2 say_red "error: could not find workflow run for PR ${PR_NUMBER}"
        exit 1
    fi
    >&2 say_white "+ Found workflow run ${WORKFLOW_RUN_ID}"

    # We need to use GO style ARCH for the artifacts
    ARCH=""
    case $(uname -m) in
        "x86_64") ARCH="amd64";;
        "arm64") ARCH="arm64";;
        "aarch64") ARCH="arm64";;
        *)
            print_unsupported_platform
            exit 1
            ;;
    esac

    # now we get the artifact URL for the user's OS and arch
    # we filter by the name of the artifact: artifacts-cli-${os}-${arch}.tar.gz
    ARTIFACT_URL=$(
        curl --retry 2 --fail --silent -L \ \
            -H "Accept: application/vnd.github+json" \
            -H "X-GitHub-Api-Version: 2022-11-28" \
            "https://api.github.com/repos/pulumi/pulumi/actions/runs/${WORKFLOW_RUN_ID}/artifacts?name=artifacts-cli-${OS}-${ARCH}" \
        | jq -r '.artifacts[0].archive_download_url')

    if [ -z "${ARTIFACT_URL}" ]; then
        >&2 say_red "error: could not find artifact URL for PR ${PR_NUMBER}"
        exit 1
    fi
    >&2 say_white "+ Found artifact URL ${ARTIFACT_URL}"

    printf "%s" "${ARTIFACT_URL}"
}

download_tarball() {
    # if the version begins with pr# then we are installing a PR build
    case $VERSION in
        # case insensitive match
        [Pp][Rr]#*)
            # try to use `gh auth login` to get a token
            if [ -z "${GITHUB_TOKEN}" ] && command -v gh >/dev/null; then
                GITHUB_TOKEN=$(gh auth token)
            fi
            if [ -z "${GITHUB_TOKEN}" ]; then
                >&2 say_red "error: GITHUB_TOKEN is required to install PR builds"
                return 1
            fi

            PR_NUMBER=$(echo "${VERSION}" | sed 's/[Pp][Rr]#//')
            ZIP_URL=$(download_github_pr "${PR_NUMBER}")
            if [ -z "${ZIP_URL}" ]; then
                return 1
            fi
            ZIP_DEST=$(mktemp -t pulumi.tar.gz.zip.XXXXXXXXXX)
            say_white "+ Downloading ${ZIP_URL} to ${ZIP_DEST}..."
            if ! curl --retry 2 --fail ${SILENT} -L -H "Authorization: Bearer ${GITHUB_TOKEN}" -o "${ZIP_DEST}" "${ZIP_URL}"; then
                return 1
            fi
            # the zip file contains a tarball named something arbitrary, like
            # pulumi-v3.129.1-alpha.1723553965-linux-x64.tar.gz
            # we won't know the exact name

            # extract the tarball from the zip
            ZIP_EXTRACT_DIR=$(mktemp -dt pulumi.zip.XXXXXXXXXX)
            say_white "+ Extracting ${ZIP_DEST} to ${ZIP_EXTRACT_DIR}..."
            unzip -q "${ZIP_DEST}" -d "${ZIP_EXTRACT_DIR}"
            # find the tarball
            TARBALL_DEST=$(find "${ZIP_EXTRACT_DIR}" -name "pulumi-*.tar.gz")
            if [ -z "${TARBALL_DEST}" ]; then
                >&2 say_red "error: could not find tarball in zip file"
                return 1
            fi
            # clean up the zip file
            rm -f "${ZIP_DEST}"
            return 0
        ;;
    esac

    # If we're installing a dev version, we need to use the s3 URL,
    # as the version is not uploaded to GitHub releases
    if [ "$IS_DEV_VERSION" = "true" ]; then
        say_white "+ Downloading ${TARBALL_URL_FALLBACK}${TARBALL_PATH}..."
        if ! curl --retry 2 --fail ${SILENT} -L -o "${TARBALL_DEST}" "${TARBALL_URL_FALLBACK}${TARBALL_PATH}"; then
            return 1
        fi
        return 0
    fi
    # Try to download from github first, then fallback to get.pulumi.com
    say_white "+ Downloading ${TARBALL_URL}${TARBALL_PATH}..."
    # This should opportunistically use the GITHUB_TOKEN to avoid rate limiting
    # ...I think. It's hard to test accurately. But it at least doesn't seem to hurt.
    if ! curl --fail ${SILENT} -L \
        --header "Authorization: Bearer $GITHUB_TOKEN" \
        -o "${TARBALL_DEST}" "${TARBALL_URL}${TARBALL_PATH}"; then
        say_white "+ Error encountered, falling back to ${TARBALL_URL_FALLBACK}${TARBALL_PATH}..."
        if ! curl --retry 2 --fail ${SILENT} -L -o "${TARBALL_DEST}" "${TARBALL_URL_FALLBACK}${TARBALL_PATH}"; then
            return 1
        fi
    fi
}

if download_tarball; then
    say_white "+ Extracting to ${PULUMI_INSTALL_ROOT}/bin"

    # If `~/.pulumi/bin` exists, remove previous files with a pulumi prefix
    if [ -e "${PULUMI_INSTALL_ROOT}/bin/pulumi" ]; then
        rm "${PULUMI_INSTALL_ROOT}/bin"/pulumi*
    fi

    mkdir -p "${PULUMI_INSTALL_ROOT}"

    # Yarn's shell installer does a similar dance of extracting to a temp
    # folder and copying to not depend on additional tar flags
    EXTRACT_DIR=$(mktemp -dt pulumi.XXXXXXXXXX)
    tar zxf "${TARBALL_DEST}" -C "${EXTRACT_DIR}"

    # Our tarballs used to have a top level bin folder, so support that older
    # format if we detect it. Newer tarballs just have all the binaries in
    # the top level Pulumi folder.
    if [ -d "${EXTRACT_DIR}/pulumi/bin" ]; then
        mv "${EXTRACT_DIR}/pulumi/bin" "${PULUMI_INSTALL_ROOT}/"
    else
        cp -r "${EXTRACT_DIR}/pulumi/." "${PULUMI_INSTALL_ROOT}/bin/"
    fi

    rm -f "${TARBALL_DEST}"
    rm -rf "${EXTRACT_DIR}"
else
    if [ "$PR_NUMBER" != "" ]; then
        >&2 say_red "error: failed to download PR ${PR_NUMBER}"
        exit 33 # skip ordinary error message
    else
        >&2 say_red "error: failed to download ${TARBALL_URL}"
        >&2 say_red "       check your internet and try again; if the problem persists, file an"
        >&2 say_red "       issue at https://github.com/pulumi/pulumi/issues/new/choose"
        exit 1
    fi
fi

# Now that we have installed Pulumi, if it is not already on the path, let's add a line to the
# user's profile to add the folder to the PATH for future sessions.
if [ "${NO_EDIT_PATH}" != "true" ] && ! command -v pulumi >/dev/null; then
    # If we can, we'll add a line to the user's .profile adding ${PULUMI_INSTALL_ROOT}/bin to the PATH
    SHELL_NAME=$(basename "${SHELL}")
    PROFILE_FILE=""

    case "${SHELL_NAME}" in
        "bash")
            # Terminal.app on macOS prefers .bash_profile to .bashrc, so we prefer that
            # file when trying to put our export into a profile. On *NIX, .bashrc is
            # preferred as it is sourced for new interactive shells.
            if [ "$(uname)" != "Darwin" ]; then
                if [ -e "${HOME}/.bashrc" ]; then
                    PROFILE_FILE="${HOME}/.bashrc"
                elif [ -e "${HOME}/.bash_profile" ]; then
                    PROFILE_FILE="${HOME}/.bash_profile"
                fi
            else
                if [ -e "${HOME}/.bash_profile" ]; then
                    PROFILE_FILE="${HOME}/.bash_profile"
                elif [ -e "${HOME}/.bashrc" ]; then
                    PROFILE_FILE="${HOME}/.bashrc"
                fi
            fi
            ;;
        "zsh")
            if [ -e "${ZDOTDIR:-$HOME}/.zshrc" ]; then
                PROFILE_FILE="${ZDOTDIR:-$HOME}/.zshrc"
            fi
            ;;
    esac

    if [ -n "${PROFILE_FILE}" ]; then
        LINE_TO_ADD="export PATH=\$PATH:${PULUMI_INSTALL_ROOT}/bin"
        if ! grep -q "# add Pulumi to the PATH" "${PROFILE_FILE}"; then
            say_white "+ Adding ${PULUMI_INSTALL_ROOT}/bin to \$PATH in ${PROFILE_FILE}"
            printf "\\n# add Pulumi to the PATH\\n%s\\n" "${LINE_TO_ADD}" >> "${PROFILE_FILE}"
        fi

        EXTRA_INSTALL_STEP="+ Please restart your shell or add ${PULUMI_INSTALL_ROOT}/bin to your \$PATH"
    else
        EXTRA_INSTALL_STEP="+ Please add ${PULUMI_INSTALL_ROOT}/bin to your \$PATH"
    fi
fi

# Warn if the pulumi command is available, but it is in a different location from the current install root.
if [ "$(command -v pulumi)" != "" ] && [ "$(command -v pulumi)" != "${PULUMI_INSTALL_ROOT}/bin/pulumi" ]; then
    say_yellow
    say_yellow "warning: Pulumi has been installed to ${PULUMI_INSTALL_ROOT}/bin, but it looks like there's a different copy"
    say_yellow "         on your \$PATH at $(dirname "$(command -v pulumi)"). You'll need to explicitly invoke the"
    say_yellow "         version you just installed or modify your \$PATH to prefer this location."
fi

say_blue
say_blue "=== Pulumi is now installed! ðŸ¹ ==="
if [ "$EXTRA_INSTALL_STEP" != "" ]; then
    say_white "${EXTRA_INSTALL_STEP}"
fi
say_green "+ Get started with Pulumi: https://www.pulumi.com/docs/quickstart"
