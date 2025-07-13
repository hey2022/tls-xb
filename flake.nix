{
  description = "Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ self, flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      perSystem =
        {
          system,
          pkgs,
          ...
        }:
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [
              inputs.fenix.overlays.default
            ];
          };
          formatter = pkgs.nixfmt-rfc-style;
          packages =
            let
              cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
            in
            {
              default = pkgs.rustPlatform.buildRustPackage {
                pname = cargoToml.package.name;
                version = "${cargoToml.package.version}+${self.lastModifiedDate}.${self.shortRev}";

                src = ./.;
                cargoLock = {
                  lockFile = ./Cargo.lock;
                };
              };
            };

          devShells = {
            default = pkgs.mkShell {
              packages = with pkgs; [
                (fenix.complete.withComponents [
                  "cargo"
                  "clippy"
                  "rust-src"
                  "rustc"
                  "rustfmt"
                ])
                openssl
                pkg-config
                cargo-deny
                cargo-edit
                cargo-watch
                rust-analyzer
                cargo-release
              ];
            };
          };
        };
    };
}
