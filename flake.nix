{
  description = "A very basic flake";

  nixConfig.extra-substituters = [
    "https://nix-community.cachix.org"
  ];

  nixConfig.extra-trusted-public-keys = [
    "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
  ];
  
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";

    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-parts, fenix, ... } @ inputs: 
  (flake-parts.lib.evalFlakeModule
  { inherit inputs; }
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


