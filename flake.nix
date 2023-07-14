{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, ... }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        name = "snow";
      in
      rec
      {
        packages.${name} = pkgs.callPackage ./default.nix { };

        # `nix build`
        defaultPackage = packages.${name}; # legacy
        packages.default = packages.${name};

        # `nix run`
        apps.${name} = utils.lib.mkApp {
          inherit name;
          drv = packages.${name};
        };
        defaultApp = apps.${name};

        devShell = pkgs.mkShell {
          buildInputs = (with pkgs; [
            cargo
            cargo-tarpaulin
            clippy
            openssl
            pkg-config
            rust-analyzer
            rustc
            rustfmt
            sqlite
          ]) ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs; [ darwin.apple_sdk.frameworks.Security libiconv ]);
        };
      });
}
