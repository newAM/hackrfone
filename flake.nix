{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    treefmt.url = "github:numtide/treefmt-nix";
    treefmt.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    nixpkgs,
    treefmt,
  }: let
    pkgs = nixpkgs.legacyPackages.x86_64-linux;

    treefmtEval = treefmt.lib.evalModule pkgs {
      projectRootFile = "flake.nix";
      programs = {
        alejandra.enable = true;
        prettier.enable = true;
        rustfmt = {
          enable = true;
          edition = (nixpkgs.lib.importTOML ./Cargo.toml).package.edition;
        };
        taplo.enable = true;
      };
    };
  in {
    devShells.x86_64-linux.default = pkgs.mkShell {
      nativeBuildInputs = [pkgs.pkg-config];
      buildInputs = [pkgs.libusb1];
    };

    formatter.x86_64-linux = treefmtEval.config.build.wrapper;

    checks.x86_64-linux = {
      formatting = treefmtEval.config.build.check self;
    };
  };
}
