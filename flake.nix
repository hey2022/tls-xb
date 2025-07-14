{
  description = "Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
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
      imports = [
        ./flake-modules/treefmt.nix
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
          packages = {
            default = pkgs.callPackage ./default.nix {
              inherit self;
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
                cargo-deny
                cargo-dist
                cargo-edit
                cargo-release
                cargo-watch
                openssl
                pkg-config
                rust-analyzer
              ];
            };
          };
        };
    };
}
