{
  description = "OrbitFM - Rust TUI File Manager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "clippy"
          ];
          targets = [ "x86_64-pc-windows-gnu" ];
        };

        rustPlatform = pkgs.makeRustPlatform {
          cargoLock.lockFile = ./Cargo.lock;
          rustc = rustToolchain;
          cargo = rustToolchain;
        };
      in
      {

        packages.default = rustPlatform.buildRustPackage {
          pname = "orbitfm";
          version = "0.1.0";
          src = ./.;

          cargoHash = "sha256-RjdigSyfr2FQ21uWWAaC39lYUqvUTuDLRLvq4oFzOSI=";

          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];
        };

        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/orbit";
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rustToolchain
            pkg-config
            cargo-edit
            cargo-watch
          ];

          buildInputs =
            with pkgs;
            [
              openssl
            ]
            ++ (lib.optional stdenv.isDarwin [
              darwin.apple_sdk.frameworks.Security
            ]);

          shellHook = ''
            echo "OrbitFM Dev Environment Loaded!"
            echo "Targets: Linux (default), Windows (x86_64-pc-windows-gnu)"
            echo "Rust Version: $(rustc --version)"
          '';
        };
      }
    );
}
