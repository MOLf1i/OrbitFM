{
  description = "OrbitFM - Rust TUI File Manager Development Environment";

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

        # Вибираємо стабільний Rust з компонентами для розробки
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "clippy"
          ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rustToolchain
            pkg-config
            cargo-edit # для швидкого додавання залежностей (cargo add)
            cargo-watch # для авто-перезапуску при зміні коду
          ];

          buildInputs =
            with pkgs;
            [
              # Бібліотеки, необхідні для роботи термінальних інтерфейсів та системних викликів
              openssl
            ]
            ++ (lib.optional stdenv.isDarwin [
              # Додаткові залежності лише для macOS, якщо захочеш розширюватись
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
