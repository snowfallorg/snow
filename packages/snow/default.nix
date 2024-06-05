{
  darwin,
  inputs,
  lib,
  makeWrapper,
  openssl,
  pkg-config,
  rustPlatform,
  sqlite,
  stdenv,
  system,
}:
rustPlatform.buildRustPackage {
  pname = "snow";
  version = "0.1.0";

  src = [ ../.. ];

  cargoLock = {
    lockFile = ../../Cargo.lock;
    outputHashes = {
      "libsnow-0.0.1-alpha.2" = "sha256-meBw64421WsoPE5Wv+h2a1AK7NLiLI45ArfIZOt2liM=";
    };
  };

  nativeBuildInputs = [
    makeWrapper
    pkg-config
  ];
  buildInputs = ([
    inputs.libsnow.packages.${system}.libsnow-helper
    openssl
    sqlite
  ]) ++ lib.optionals stdenv.isDarwin [ darwin.apple_sdk.frameworks.Security ];

  doCheck = false;

  postInstall = ''
    wrapProgram $out/bin/snow --prefix PATH : '${"${inputs.libsnow.packages.${system}.libsnow-helper}/libexec:${inputs.libsnow.packages.${system}.libsnow-helper}/share/libsnow/triggers"}'
  '';
}
