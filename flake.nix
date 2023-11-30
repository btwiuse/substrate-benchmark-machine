{
  description = "Example cross-compiling rust w/ openssl for aarch64-unknown-linux-musl";

  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  outputs = {
    self,
    nixpkgs,
  }: let
    system = "x86_64-linux";
    musl = "aarch64-unknown-linux-musl";
    pkgs = import nixpkgs {
      inherit system;
      crossSystem = {
        config = musl;
        rustc.config = musl;
        isStatic = true;
      };
    };
  in {
    packages.x86_64-linux.default = pkgs.rustPlatform.buildRustPackage {
      name = "substrate-benchmark-machine";
      version = "0.0";
      src = ./.;
      cargoLock.lockFile = ./Cargo.lock;

      nativeBuildInputs = with pkgs; [
      # pkg-config
      ];
      buildInputs = with pkgs; [
      # openssl
      ];
    };
  };
}
