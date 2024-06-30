{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    snowfall-lib = {
      url = "github:snowfallorg/lib/v3.0.3";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    libsnow = {
      url = "github:snowfallorg/libsnow";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs:
    inputs.snowfall-lib.mkFlake {
      inherit inputs;
      alias.packages.default = "snow";
      alias.shells.default = "snow";
      src = ./.;
    };
}
