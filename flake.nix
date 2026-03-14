{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    flake-utils.url = "github:numtide/flake-utils";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    helper.url = "github:m-lima/nix-template";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      helper,
      ...
    }@inputs:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        sharedOptions = {
          allowFilesets = [
            ./store/.sqlx
            ./store/migrations
          ];
          systemLinker = pkgs.stdenv.isLinux;
          formatters = {
            beautysh.enable = true;
            prettier = {
              enable = true;
              settings = builtins.fromJSON (builtins.readFile ./web/.prettierrc.json);
            };
          };
          fmtExcludes = [
            "*.png"
            "*.sqlite"
            "*.txt"
            "*Dockerfile"
            ".dockerignore"
            "store/.sqlx/*.json"
            "store/migrations/*.sql"
          ];
          buildInputs = pkgs: [ pkgs.openssl ];
          nativeBuildInputs = pkgs: [ pkgs.pkg-config ];
          devPackages = pkgs: [
            (pkgs.writeShellScriptBin "sqlite" "exec ${pkgs.sqlite}/bin/sqlite3 -init ${pkgs.writeText "sqliteconfig" ".mode columns"} $@")
            pkgs.sqlx-cli
            pkgs.yarn
          ];
          overrides.commonArgs.LIBSQLITE3_FLAGS = "-DSQLITE_ENABLE_MATH_FUNCTIONS=1";
        };
        all = helper.lib.rust.helper inputs system ./. sharedOptions;
        server = helper.lib.rust.helper inputs system ./. (sharedOptions // { package = "-p server"; });
        cli = helper.lib.rust.helper inputs system ./. (sharedOptions // { cargoExtraArgs = "-p cli"; });
        commonWeb = {
          nodejs = pkgs.nodejs;

          src = pkgs.lib.fileset.toSource {
            root = ./web;
            fileset = pkgs.lib.fileset.unions [
              ./web/package.json
              ./web/public
              ./web/src
              ./web/tsconfig.json
              ./web/yarn.lock
            ];
          };

          doDist = false;
        };
        web = pkgs.mkYarnPackage (
          commonWeb
          // {
            nativeBuildInputs = [ pkgs.writableTmpDirAsHomeHook ];

            buildPhase = ''
              runHook preBuild
              yarn --offline build
              runHook postBuild
            '';

            installPhase = ''
              runHook preInstall
              mv deps/$pname/build $out
              runHook postInstall
            '';
          }
        );
      in
      all.outputs
      // {
        packages = {
          server = server.outputs.packages.default;
          cli = cli.outputs.packages.default;
          web = web;
        };
        checks = all.checks // {
          webCheck = pkgs.mkYarnPackage (
            commonWeb
            // {
              dontBuild = true;

              checkPhase = ''
                runHook preBuild
                cd deps/$pname
                yarn eslint src/
                runHook postBuild
              '';

              installPhase = "mkdir -p $out";
            }
          );
        };
        apps = {
          server = server.outputs.apps.default;
          cli = cli.outputs.apps.default;
        };
      }
    );
}
