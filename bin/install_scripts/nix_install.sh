#!/bin/sh

# This script installs the Nix package manager on your system by
# downloading a binary distribution and running its installer script
# (which in turn creates and populates /nix).

{ # Prevent execution if this script was only partially downloaded
oops() {
    echo "$0:" "$@" >&2
    exit 1
}

umask 0022

tmpDir="$(mktemp -d -t nix-binary-tarball-unpack.XXXXXXXXXX || \
          oops "Can't create temporary directory for downloading the Nix binary tarball")"
cleanup() {
    rm -rf "$tmpDir"
}
trap cleanup EXIT INT QUIT TERM

require_util() {
    command -v "$1" > /dev/null 2>&1 ||
        oops "you do not have '$1' installed, which I need to $2"
}

case "$(uname -s).$(uname -m)" in
    Linux.x86_64)
        hash=3c0779e4878d1289cf3fbb158ec5ea9bdf61dfb9b4efac6b3b0b6bec5ba4cf13
        path=0xf66gpzcg4924nkfz7cn4ynqrxcfglq/nix-2.24.9-x86_64-linux.tar.xz
        system=x86_64-linux
        ;;
    Linux.i?86)
        hash=f76d680e4aa6fb03c390e824e69e3348913ff0a65d1065ee1e8049439a0039c3
        path=2hgnmrfcsi6910rb647r7aamrn6a8wz3/nix-2.24.9-i686-linux.tar.xz
        system=i686-linux
        ;;
    Linux.aarch64)
        hash=c57c2830bb407e02dacdf2b63c49cde273f905075b579f6d9a6114c669301f33
        path=6wjg39ajzfmg0v5hz31ispz74lf59c72/nix-2.24.9-aarch64-linux.tar.xz
        system=aarch64-linux
        ;;
    Linux.armv6l)
        hash=e07ebd6c641e187794454622076faf8c2286878fa7ed373ca8f39454643591e8
        path=32508s5pz33bz9cyj8nn0fschgi87gyg/nix-2.24.9-armv6l-linux.tar.xz
        system=armv6l-linux
        ;;
    Linux.armv7l)
        hash=c7df074b66026195c1c8b47024add16dba2cc5ba26d3e03ec17ccd040a0094b2
        path=pxc3i3hwg33wcrcxjyagxrzk2v5jc869/nix-2.24.9-armv7l-linux.tar.xz
        system=armv7l-linux
        ;;
    Linux.riscv64)
        hash=57ebe19c3b85079ca7d66fad4e77894abf539972ee8425246b7b67a69e8c6b48
        path=jzrvwsllk1f54s02v9vf68kvypi7chhk/nix-2.24.9-riscv64-linux.tar.xz
        system=riscv64-linux
        ;;
    Darwin.x86_64)
        hash=b0cfcc873a16aa88e0e7dc1496d49c37c3d35fb325624c487c082b89a7f1ba3f
        path=vx8hnp9m2in2nqy131929kf8ncnx8mpz/nix-2.24.9-x86_64-darwin.tar.xz
        system=x86_64-darwin
        ;;
    Darwin.arm64|Darwin.aarch64)
        hash=15b85046fb7de93078878bbcdd75f68514dd985a68bf96d0ee63b0fd58851b61
        path=nvdmlfljaaa0xpq0559ijajvpfgdvz89/nix-2.24.9-aarch64-darwin.tar.xz
        system=aarch64-darwin
        ;;
    *) oops "sorry, there is no binary distribution of Nix for your platform";;
esac

# Use this command-line option to fetch the tarballs using nar-serve or Cachix
if [ "${1:-}" = "--tarball-url-prefix" ]; then
    if [ -z "${2:-}" ]; then
        oops "missing argument for --tarball-url-prefix"
    fi
    url=${2}/${path}
    shift 2
else
    url=https://releases.nixos.org/nix/nix-2.24.9/nix-2.24.9-$system.tar.xz
fi

tarball=$tmpDir/nix-2.24.9-$system.tar.xz

require_util tar "unpack the binary tarball"
if [ "$(uname -s)" != "Darwin" ]; then
    require_util xz "unpack the binary tarball"
fi

if command -v curl > /dev/null 2>&1; then
    fetch() { curl --fail -L "$1" -o "$2"; }
elif command -v wget > /dev/null 2>&1; then
    fetch() { wget "$1" -O "$2"; }
else
    oops "you don't have wget or curl installed, which I need to download the binary tarball"
fi

echo "downloading Nix 2.24.9 binary tarball for $system from '$url' to '$tmpDir'..."
fetch "$url" "$tarball" || oops "failed to download '$url'"

if command -v sha256sum > /dev/null 2>&1; then
    hash2="$(sha256sum -b "$tarball" | cut -c1-64)"
elif command -v shasum > /dev/null 2>&1; then
    hash2="$(shasum -a 256 -b "$tarball" | cut -c1-64)"
elif command -v openssl > /dev/null 2>&1; then
    hash2="$(openssl dgst -r -sha256 "$tarball" | cut -c1-64)"
else
    oops "cannot verify the SHA-256 hash of '$url'; you need one of 'shasum', 'sha256sum', or 'openssl'"
fi

if [ "$hash" != "$hash2" ]; then
    oops "SHA-256 hash mismatch in '$url'; expected $hash, got $hash2"
fi

unpack=$tmpDir/unpack
mkdir -p "$unpack"
tar -xJf "$tarball" -C "$unpack" || oops "failed to unpack '$url'"

script=$(echo "$unpack"/*/install)

[ -e "$script" ] || oops "installation script is missing from the binary tarball!"
export INVOKED_FROM_INSTALL_IN=1
"$script" "$@"

} # End of wrapping