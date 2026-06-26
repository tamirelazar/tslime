{
  description = "A lightweight terminal screensaver simulating slime mold growth patterns";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        # Single source of truth for the version: read it from Cargo.toml so the
        # flake never drifts from the crate.
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "tslime";
          version = cargoToml.package.version;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          # Default (terminal) features only — no C/system dependencies. The gui
          # and audio features are intentionally not built here.
          #
          # Tests are skipped in the Nix sandbox: the visual-regression golden
          # tests are platform-sensitive (temporal OKLch output differs across
          # targets) and are gated in CI, which is their source of truth.
          doCheck = false;
          meta = {
            description = "A lightweight terminal screensaver simulating slime mold growth patterns";
            homepage = "https://github.com/tamirelazar/tslime";
            license = pkgs.lib.licenses.mit;
            mainProgram = "tslime";
          };
        };

        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
        };
      });
}
