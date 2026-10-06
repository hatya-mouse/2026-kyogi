{
  description = "Procon 2026 Solver";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
  let
    systems = [ "x86_64-linux" "aarch64-darwin" "x86_64-darwin" ];
  in {
    devShells = nixpkgs.lib.genAttrs systems (system:
      let
        pkgs = import nixpkgs { inherit system; };
        inherit (pkgs) lib;
        isLinux = pkgs.stdenv.hostPlatform.isLinux;
        isDarwin = pkgs.stdenv.hostPlatform.isDarwin;
      in {
        default = pkgs.mkShell ({
          packages = with pkgs; [
            cargo
            rustfmt
            clippy
          ];
        });
      }
    );
  };
}
