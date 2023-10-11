{
  description = "A rust library implementing the Hanover flipdot display protocol.";

  inputs = {
    utils.url = "github:numtide/flake-utils";
    devshell.url = "github:numtide/devshell";
    naersk.url = "github:nix-community/naersk";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    naersk,
    devshell,
    rust-overlay,
  }:
    utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };
      rust = pkgs.rust-bin.stable.latest.default;
      # Override naersk to use our chosen rust version from rust-overlay
      naersk-lib = naersk.lib.${system}.override {
        cargo = rust;
        rustc = rust;
      };
    in {
      devShells.default = let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [devshell.overlays.default];
        };
      in
        pkgs.devshell.mkShell {
          packages = with pkgs; [(rust.override {extensions = ["rust-src"];}) rust-analyzer];
        };
      formatter = pkgs.alejandra;
    });
}
