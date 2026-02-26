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
      flake-utils,
      helper,
      ...
    }@inputs:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        sharedOptions = {
          allowFilesets = [
            ./store/.sqlx
            ./store/migrations
          ];
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
          devPackages = pkgs: [ pkgs.sqlx-cli ];
        };
        # Need to override the commonArgs here because the integration tests will fail to load libssl
        # The tls feature is only needed in `cli`, however `server` uses reqwest without tls for tests
        # During a `nix flake check` calling the entire workspace, would compile the union of the
        # features and require opessl, which is not available and neither is the test executable patched
        all = helper.lib.rust.helper inputs system ./. sharedOptions;
        server = helper.lib.rust.helper inputs system ./. (
          sharedOptions // { overrides.commonArgs.cargoExtraArgs = "-p server"; }
        );
        cli = helper.lib.rust.helper inputs system ./. (
          sharedOptions // { overrides.commonArgs.cargoExtraArgs = "-p cli"; }
        );
      in
      all.outputs
      // {
        checks =
          let
            prefixAttrs =
              prefix: attrs:
              builtins.listToAttrs (
                map (k: {
                  name = "${prefix}-${k}";
                  value = attrs.${k};
                }) (builtins.attrNames attrs)
              );
          in
          (prefixAttrs "server" server.checks) // (prefixAttrs "cli" cli.checks);
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
