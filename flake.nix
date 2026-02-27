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
          };
          fmtExcludes = [
            "Dockerfile"
            ".dockerignore"
            "store/.sqlx/*.json"
            "store/migrations/*.sql"
          ];
          buildInputs = pkgs: [ pkgs.openssl ];
          nativeBuildInputs = pkgs: [ pkgs.pkg-config ];
          devPackages = pkgs: [
            (pkgs.writeShellScriptBin "sqlite" "exec ${pkgs.sqlite}/bin/sqlite3 -init ${pkgs.writeText "sqliteconfig" ".mode columns"} $@")
            pkgs.git-crypt
            pkgs.sqlx-cli
            pkgs.yarn
          ];
        };
        all = helper.lib.rust.helper inputs system ./. sharedOptions;
        server = helper.lib.rust.helper inputs system ./. (
          sharedOptions // { overrides.mainArgs.cargoExtraArgs = "-p server"; }
        );
        cli = helper.lib.rust.helper inputs system ./. (
          sharedOptions // { overrides.mainArgs.cargoExtraArgs = "-p cli"; }
        );
      in
      all.outputs
      // {
        packages = {
          server = server.outputs.packages.default;
          cli = cli.outputs.packages.default;
        };
        apps = {
          server = server.outputs.apps.default;
          cli = cli.outputs.apps.default;
        };
      }
    );
}
