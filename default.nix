{ pkgs ? import <nixpkgs> { }
, lib ? import <nixpkgs/lib>
}:
pkgs.rustPlatform.buildRustPackage {
  pname = "snow";
  version = "0.0.2";

  src = [ ./. ];

  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "nix-data-0.0.3" = "sha256-kLcAtvZPa1VKHmMJR3xiX94lkkmfUFvzn/pnw6r5w4I=";
    };
  };

  nativeBuildInputs = with pkgs; [
    makeWrapper
    pkg-config
  ];
  buildInputs = (with pkgs; [
    openssl
  ]) ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs; [ darwin.apple_sdk.frameworks.Security ]);

  doCheck = false;

  postInstall = ''
    wrapProgram $out/bin/snow --prefix PATH : '${pkgs.lib.makeBinPath [ pkgs.sqlite ]}'
    mkdir -p $out/libexec
    mv $out/bin/snow-helper $out/libexec/snow-helper
  '';
}
