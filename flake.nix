{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, flake-utils, naersk, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        naersk-lib = naersk.lib."${system}";
      in
      {
        packages.snow = naersk-lib.buildPackage {
          pname = "snow";
          root = ./.;
          nativeBuildInputs = with pkgs; [ makeWrapper ];
          buildInputs = with pkgs; [
            openssl
            pkg-config
            sqlite
          ];
          postInstall = ''
            wrapProgram $out/bin/snow --prefix PATH : '${pkgs.lib.makeBinPath [ pkgs.sqlite ]}'
            mkdir -p $out/libexec
            mv $out/bin/snow-helper $out/libexec/snow-helper
          '';
        };

        defaultPackage = self.packages.${system}.snow;

        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            cargo-tarpaulin
            clippy
            openssl
            pkg-config
            rust-analyzer
            rustc
            rustfmt
            sqlite
          ];
        };
      });
}
