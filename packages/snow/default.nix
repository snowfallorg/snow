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
}:
rustPlatform.buildRustPackage {
  pname = "snow";
  version = "0.2.0";

  src = [ ../.. ];

  cargoLock = {
    lockFile = ../../Cargo.lock;
    outputHashes = {
      "libsnow-0.0.2-alpha.1" = "sha256-PjEehXjy1SdqN28Z3QUDr8YHqd/J024NIwuNDhYHwAY=";
    };
  };

  nativeBuildInputs = [
    makeWrapper
    pkg-config
  ];
  buildInputs = ([
    inputs.libsnow.packages.${stdenv.hostPlatform.system}.libsnow-helper
    openssl
    sqlite
  ])
  ++ lib.optionals stdenv.isDarwin [ darwin.apple_sdk.frameworks.Security ];

  doCheck = false;

  postInstall = ''
    wrapProgram $out/bin/snow --prefix PATH : '${"${inputs.libsnow.packages.${stdenv.hostPlatform.system}.libsnow-helper}/libexec:${inputs.libsnow.packages.${stdenv.hostPlatform.system}.libsnow-helper}/share/libsnow/triggers"}'
  '';
}
