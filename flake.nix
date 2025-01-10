{
  description = "A beholder for data";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
  };
  outputs =
    { self, nixpkgs }:
    let
      supportedSystems = [
        "x86_64-darwin"
        "x86_64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      pkgsFor = nixpkgs.legacyPackages;
    in
    {
      packages = forAllSystems (system: {
        default = pkgsFor.${system}.rustPlatform.buildRustPackage {
          pname = "beholder";
          version = "0.1.0";
          cargoLock.lockFile = ./Cargo.lock;
          src = nixpkgs.lib.cleanSource ./.;
          buildInputs = [ pkgsFor.${system}.postgresql_17 ];
        };
      });
    };
}
