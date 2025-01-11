{
  pkgs ? import <nixpkgs> { },
}:
pkgs.rustPlatform.buildRustPackage {
  pname = "beholder";
  version = "0.1.0";
  cargoLock.lockFile = ./Cargo.lock;
  src = pkgs.lib.cleanSource ./.;
  buildInputs = [
    pkgs.postgresql_17
    pkgs.openssl
  ];
  nativeBuildInputs = [
    pkgs.pkg-config
  ];
}
