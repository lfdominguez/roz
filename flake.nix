{
  description = "roz ollama cli";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";

    crane = {
        url = "github:ipetkov/crane";
        inputs = {
            nixpkgs.follows = "nixpkgs";
        };
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
            overlays = [ (import rust-overlay) ];

            pkgs = import nixpkgs {
                inherit system overlays;
            };

            rustToolchain = pkgs.rust-bin.stable.latest.default;

            craneLib = crane.lib.${system};

            src = craneLib.cleanCargoSource ./.;

            nativeBuildInputs = with pkgs; [
                rustToolchain
                pkg-config
            ];

            buildInputs = with pkgs; [
                openssl
            ];

            commonArgs = {
                inherit src buildInputs nativeBuildInputs;

                cargoTestCommand = "echo 'Ignoring tests'";
            };

            cargoArtifacts = craneLib.buildDepsOnly commonArgs;

            bin = craneLib.buildPackage (commonArgs // {
                inherit cargoArtifacts;
            });

            dockerImage = pkgs.dockerTools.buildLayeredImage {
                name = "roz";

                content = pkgs.buildEnv {
                    name = "roz";
                    paths = [ bin ];
                    pathsToLink = [ "/bin" ];
                };
 
                config = {
                  Cmd = [ "/bin/roz" ];
                };
            };
        in
        with pkgs;
        {
            packages = {
                inherit bin dockerImage;
                default = bin;
            };

            devShells.default = mkShell {
                inputsFrom = [ bin ];
                buildInputs = [ dive ];
            };
        }
    );
}
