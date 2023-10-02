{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, flake-utils, naersk, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk' = pkgs.callPackage naersk {};

      in rec {
        # For `nix build` & `nix run`:
        defaultPackage = naersk'.buildPackage {
          nativeBuildInputs = with pkgs; [];
          src = ./.;
        };

        # For `nix develop`:
        devShell = pkgs.mkShell {
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          nativeBuildInputs =
            let
              univPkgs = with pkgs; [
                  # Build requirements
                  rustc
                  cargo
                  libiconv
                  # Extras
                  rust-analyzer
                  rustfmt
                  bacon
                  cargo-watch
                  clippy
                  diesel-cli
                ];
              darwinPkgs = with pkgs; [
                  darwin.apple_sdk.frameworks.Security
                ];
            in
              univPkgs ++  (if pkgs.stdenv.isDarwin then darwinPkgs else []);
        };
      }
    );
}
