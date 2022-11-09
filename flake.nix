{
  description = "A very basic flake";
  
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";

    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";

    fenix = {
      url = "github:nix-community/fenix/b3e5ce9985c380c8fe1b9d14879a14b749d1af51";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-parts, fenix, ... } @ inputs: 
  (flake-parts.lib.evalFlakeModule
  { inherit self; }
  {
    systems = ["x86_64-linux" "aarch64-linux" "aarch64-darwin"];
    imports = [
    ];
    perSystem = {system, self', inputs', pkgs, ...}: let
      fenixPkgs = fenix.packages.${system};
      toolchain = fenixPkgs.stable;
      rustToolchain = fenixPkgs.combine [
        toolchain.cargo
        toolchain.rustc
        toolchain.rust-src
        toolchain.rust-std
        toolchain.clippy
        toolchain.rustfmt
        #targets.x86_64-unknown-linux-musl.stable.rust-std
        # fenix.packages.x86_64-linux.targets.aarch64-unknown-linux-gnu.latest.rust-std
      ];

      rustPlatform = (pkgs.makeRustPlatform {
        cargo = rustToolchain;
        rustc = rustToolchain;
      });
    in {
      devShells.default = with pkgs; mkShell {
        RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        buildInputs = [
          rustToolchain
          fenixPkgs.rust-analyzer
          just
        ];
      };
      packages.default = rustPlatform.buildRustPackage {
        name = "hosthog";
        src = self;
        buildInputs = [];
        cargoLock = {
          lockFile = ./Cargo.lock;
          outputHashes = {
          };
        };
      };
    };
  }).config.flake;
}


