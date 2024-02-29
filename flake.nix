{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default;
        craneLib = crane.lib.${system}.overrideToolchain rustToolchain;

        # common source and build dependencies
        isDarwin = pkgs.lib.strings.hasSuffix "-darwin" system;
        commonInputs = {
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          nativeBuildInputs = with pkgs; [
            rustToolchain
            pkg-config
          ] ++ lib.optional (isDarwin) darwin.apple_sdk.frameworks.SystemConfiguration;
          # buildInputs = with pkgs; [ openssl sqlite ];
        };

        # build the rust app
        bin = craneLib.buildPackage commonInputs // {
          cargoArtifacts = craneLib.buildDepsOnly commonInputs;
        };
      in
      with pkgs;
      {
        packages = {
          default = bin;
        };
        devShells.default = mkShell {
          inputsFrom = [ bin ];
        };
        nixosModules.default = { self, config, lib, pkgs, ... }:
          let
            cfg = config.services.derper-verifier;
            writeTrustedClients = clients:
              pkgs.writeText "trusted-clients"
                (builtins.concatStringsSep "\n" clients);
          in
          with lib;
          {
            options.services.derper-verifier = {
              enable = mkOption {
                default = false;
                description = "Enable the derper-verifier systemd service";
              };
              trustedClients = mkOption {
                default = "";
                description = "A list of trusted clients, one per line";
              };
            };

            config.systemd.services.derper-verifier = mkIf cfg.enable {
              description = "A verifier for the --verify-clients-url feature of custom Tailscale derpers";
              wantedBy = [ "multi-user.target" ];
              serviceConfig = {
                Type = "simple";
                ExecStart = "${bin}/bin/derper-verifier";
                Restart = "always";
                RestartSec = "30";
                DynamicUser = true;
                Environment = [
                  "RUST_LOG=info"
                  # TODO: Make these options
                  "DERPER_VERIFIER_ADDR=127.0.0.1"
                  "DERPER_VERIFIER_ADDR=3000"
                  "DERPER_VERIFIER_CONFIG=${writeTrustedClients cfg.trustedClients}"
                ];
              };
            };
          };
      });
}
