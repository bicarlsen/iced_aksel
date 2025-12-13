# Thanks to: https://fasterthanli.me/series/building-a-rust-service-with-nix/part-10#a-flake-with-a-dev-shell
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem
    (
      system: let
        overlays = [rust-overlay.overlays.default];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = pkgs.pkgsBuildHost.rust-bin.stable.latest.default.override {
          targets = [
            "wasm32-unknown-unknown"
            "x86_64-pc-windows-msvc"
            "x86_64-unknown-linux-gnu"
            "x86_64-apple-darwin"
          ]; # Make sure we have all targets installed
        };
      in
        with pkgs; {
          devShells.default = mkShell rec {
            buildInputs = with pkgs;
              [
                # Programs/Addons
                bacon
                cargo-edit
                cargo-expand
                cargo-nextest
                cargo-machete
                chromedriver
                dbus
                expat
                fontconfig
                freetype
                just
                pkg-config
                trunk
                sqlite
                wasm-bindgen-cli
                wasm-pack

                # Libraries
                m4 # REQUIRED for GMP/MPFR build
                gmp # optional: system GMP, faster builds
                mpfr # optional: system MPFR
                freetype.dev
                libGL
                libxkbcommon
                openssl
              ]
              # Rust stuff (Cargo, rust-analyzer, rustfmt, clippy, etc.)
              ++ [rustToolchain]
              # If on linux
              ++ lib.optionals stdenv.isLinux [
                alsa-lib
                wayland
                xorg.libX11
                xorg.libXcursor
                xorg.libXi
                xorg.libXrandr
                vulkan-loader
              ];

            LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;

            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          };
        }
    );
}
