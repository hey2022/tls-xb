{ inputs, ... }:

{
  imports = [ inputs.treefmt-nix.flakeModule ];
  perSystem = {
    treefmt = {
      projectRootFile = "flake.nix";
      programs = {
        nixfmt.enable = true;
        rustfmt.enable = true;
        prettier = {
          enable = true;
          excludes = [
            ".github/workflows/release.yml" # auto generated and checked by CI
          ];
        };
      };
    };
  };
}
