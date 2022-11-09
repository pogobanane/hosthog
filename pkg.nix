{ rustPlatform, pkgs, pkgSrc }:
rustPlatform.buildRustPackage {
  name = "hosthog";
  src = pkgSrc;
  buildInputs = [];
  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
    };
  };
}
