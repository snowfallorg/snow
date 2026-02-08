<div align="center">

Snow
===
[![Built with Nix][builtwithnix badge]][builtwithnix]
[![License: MIT][MIT badge]][MIT]

:warning: Snow **only** works on nix flakes based systems :warning:

</div>

Snow is a small command-line tool that allows you to configure all your NixOS packages in one place. Snow is written with rust and uses [libsnow](https://github.com/snowfallorg/libsnow) and [nix-editor](https://github.com/snowfallorg/nix-editor) to parse and edit configuration files.


# Installation
```bash
nix profile install github:snowfallorg/snow
```

# Run without installing
```bash
nix run github:snowfallorg/snow
```

# Usage

## Install a package
```
Usage: snow install [OPTIONS] [PACKAGES]...

Arguments:
  [PACKAGES]...  

Options:
  -s, --system  
  -h, --home    
  -h, --help    Print help
```

## Remove a package
```
Usage: snow remove [OPTIONS] [PACKAGES]...

Arguments:
  [PACKAGES]...  

Options:
  -s, --system  
  -h, --home    
  -h, --help    Print help
```

## Update package/s
```
Usage: snow update [OPTIONS] [PACKAGES]...

Arguments:
  [PACKAGES]...  

Options:
  -s, --system  
  -h, --home    
  -a, --all     
  -h, --help    Print help
```

## Rebuild system configuration
```
Usage: snow rebuild

Options:
  -h, --help  Print help information
```

## List installed packages
```
Usage: snow list [OPTIONS]

Options:
  -p, --profile  
  -s, --system   
  -h, --help     Print help information
```

## Search for a package
```
Usage: snow search [QUERY]...

Arguments:
  [QUERY]...  

Options:
  -h, --help  Print help information
```

## Run a package not currently installed
```
Usage: snow run <PACKAGE> [ARGUMENTS]...

Arguments:
  <PACKAGE>       
  [ARGUMENTS]...  

Options:
  -h, --help  Print help information
```

[builtwithnix badge]: https://img.shields.io/badge/Built%20With-Nix-41439A?style=for-the-badge&logo=nixos&logoColor=white
[builtwithnix]: https://builtwithnix.org/
[MIT badge]: https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge
[MIT]: https://opensource.org/licenses/MIT
