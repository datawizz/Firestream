{
  lib,
  rustPlatform,
  pkg-config,
  stdenv,
  openssl,
  zlib,
  zstd,
  bzip2,
  xz,
  libiconv,
  llvmPackages,
}:

rustPlatform.buildRustPackage {
  pname = "firestream";
  version = "0.1.0";

  src = ./..;

  cargoLock.lockFile = ./../Cargo.lock;

  nativeBuildInputs = [
    pkg-config
    llvmPackages.clang
  ];

  buildInputs = [
    openssl
    openssl.dev
    zlib
    zstd
    bzip2
    xz
  ] ++ lib.optionals stdenv.hostPlatform.isDarwin [
    libiconv
  ];

  env.LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";

  # Build only the CLI binary from the workspace
  buildAndTestSubdir = "src/lib/rust/firestream";

  doCheck = false;

  meta = {
    description = "CLI/TUI tool for managing data infrastructure services";
    homepage = "https://github.com/datawizz/Firestream";
    license = lib.licenses.mit;
    maintainers = [];
    mainProgram = "firestream";
    platforms = lib.platforms.unix;
  };
}
