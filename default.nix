{
  self,
  lib,
  rustPlatform,
}:

let
  cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
in
rustPlatform.buildRustPackage rec {
  pname = cargoToml.package.name;
  version = "${cargoToml.package.version}+${self.lastModifiedDate}.${self.shortRev}";

  src = ./.;
  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  meta = {
    description = "CLI tool that fetches scores and GPA from https://tsinglanstudent.schoolis.cn";
    homepage = "https://github.com/hey2022/tls-xb";
    license = lib.licenses.gpl3Only;
    mainProgram = "tls-xb";
  };
}
