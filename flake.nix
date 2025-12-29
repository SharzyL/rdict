{
  inputs = {
    nixpkgs.url = "nixpkgs";
    flake-parts.url = "flake-parts";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { flake-parts, ... }@inputs:
    let
      name = "rdict";
      makePkg =
        { lib
        , rustPlatform
        , pkg-config
        , openssl
        , wayland
        , wayland-protocols
        , libxkbcommon
        , libGL
        , fontconfig
        }:
        rustPlatform.buildRustPackage rec {
          inherit name;

          nativeBuildInputs = [
            pkg-config
          ];

          buildInputs = [
            openssl
            fontconfig
          ];

          passthru.lib_path = lib.makeLibraryPath [
            wayland
            wayland-protocols
            libxkbcommon
            libGL
          ];

          src = with lib.fileset; toSource {
            root = ./.;
            fileset = fileFilter
              (file: ! (lib.elem file.name [ "flake.nix" "flake.lock" ]))
              ./.;
          };

          postFixup = ''
            patchelf --add-rpath ${passthru.lib_path} $out/bin/${name}
          '';

          cargoHash = "sha256-U4rtaw64UalVaPHF5wOvKmMtRvC3XmLXkr1K7E+KnTE=";
          meta.mainProgram = name;
        };

      shellOverride = pkgs: oldAttrs: {
        name = "${name}-dev-shell";
        version = null;
        src = null;

        # https://github.com/NixOS/nixpkgs/issues/214945
        nativeBuildInputs = (oldAttrs.nativeBuildInputs or [ ]) ++ (with pkgs; [
          clippy
        ]);

        shellHook = ''
          export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${oldAttrs.passthru.lib_path}"
        '';
      };

      overlay = final: _: { ${name} = final.callPackage makePkg { }; };

    in
    # flake-parts boilerplate
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.treefmt-nix.flakeModule
      ];

      flake.overlays.default = overlay;

      systems = inputs.nixpkgs.lib.systems.flakeExposed;

      perSystem = { system, config, pkgs, ... }: {
        packages.default = config.legacyPackages.${name};
        packages.${name} = config.packages.default;
        legacyPackages = pkgs;

        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [ overlay ];
        };

        devShells.default = config.packages.default.overrideAttrs (shellOverride pkgs);

        treefmt = {
          programs.rustfmt.enable = true;
          programs.nixpkgs-fmt.enable = true;
        };
      };
    };
}
