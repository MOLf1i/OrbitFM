{
  description = "OrbitFM - Rust TUI File Manager Development Environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
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
      in
      {
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
              pkgs.mingw_w64
              openssl
            ]
            ++ (lib.optional stdenv.isDarwin [
              darwin.apple_sdk.frameworks.Security
            ]);

          shellHook = ''
            echo "OrbitFM Dev Environment Loaded!"
            echo "Rust Version: $(rustc --version)"
          '';
        };
      }
    );
}
