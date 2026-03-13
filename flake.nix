{

  inputs = {
    self.submodules = true;

    nixpkgs.url = "nixpkgs/nixos-unstable";

    determinate-nix = {
      url = "github:DeterminateSystems/nix-src";
    };

    crane.url = "github:ipetkov/crane";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "determinate-nix/nixpkgs";
    };

  };
  outputs =
    {
      nixpkgs,
      crane,
      rust-overlay,
      determinate-nix,
      ...
    }@inputs:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      };
      inherit (pkgs) lib;

      target = "wasm32-unknown-unknown";
      craneLib = (crane.mkLib pkgs).overrideToolchain (
        p:
        p.rust-bin.stable.latest.default.override {
          targets = [ target ];
        }
      );
    in

    let
      wasm-nix = import ./wasm.nix { inherit pkgs craneLib target; };
      dnix = pkgs.writeShellScriptBin "dnix" "${
        lib.getExe determinate-nix.packages.${system}.default
      } $@";
    in
    rec {
      packages.${system} = rec {
        inherit dnix;
        wasm = pkgs.buildEnv {
          name = "nix-wasm";
          paths = builtins.attrValues wasm-nix.wasm;
        };
        default = wasm;
      }
      // wasm-nix.wasm;

      checks.${system} =
        with builtins;
        with lib;
        let
          testNix = {
            a = 1;
            b = "b";
            c = [
              "d"
              2
            ];
            f = {
              g = 3;
            };
          };
          runTest =
            name:
            (import ./tests/${name}) {
              inherit testNix;
              out = packages.${system}.wasm;
              drv =
                result:
                derivation {
                  inherit system name;
                  builder = lib.getExe pkgs.bash;
                  args = [
                    "-c"
                    ''echo -e "${name}:\n${result}" && echo "" > $out''
                  ];
                };
            };
          tests = attrNames (
            mapAttrs (name: type: type == "regular" && hasSuffix ".nix" name) (readDir ./tests)
          );
        in
        listToAttrs (
          map (test: {
            name = test;
            value = runTest test;
          }) tests
        );

      devShells.${system}.default = craneLib.devShell {
        packages = [
          wasm-nix.wasm-ld
          dnix
        ]
        ++ (with pkgs; [
          cargo-expand
        ]);
      };
    };
}
