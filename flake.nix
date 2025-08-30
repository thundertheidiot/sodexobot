{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    crane,
    ...
  }: let
    inherit (nixpkgs) lib;

    forAllSystems = fn:
      lib.genAttrs lib.systems.flakeExposed
      (system:
        fn
        (import nixpkgs {
          inherit system;
          overlays = [
            (import rust-overlay)
          ];
        }));
  in {
    devShells = forAllSystems (pkgs: let
      toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
    in {
      default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [pkg-config openssl];

        buildInputs = with pkgs; [
          toolchain
        ];
      };
    });

    packages = forAllSystems (pkgs: let
      toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
    in rec {
      default = craneLib.buildPackage rec {
        inherit (cargoToml.package) version;
        pname = cargoToml.package.name;
        src = ./.;

        nativeBuildInputs = with pkgs; [
          pkg-config
          openssl
        ];
      };

      docker = pkgs.dockerTools.buildLayeredImage {
        name = "ghcr.io/thundertheidiot/sodexobot";
        tag = "latest";

        contents = [default pkgs.cacert];

        config = {
          Cmd = "/bin/sodexobot";
          Labels = {
            "org.opencontainers.image.source" = "https://github.com/thundertheidiot/sodexobot";
          };
        };
      };
    });
  };
}
