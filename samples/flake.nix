{
  description = "NixOS configuration";

  inputs = {
    # Add the keepass-2-file input
    keepass-2-file.url = "github:Dracks/keepass-2-file/feat/adding-nix";
  };

  outputs =
    inputs@{
      keepass-2-file,
      ...
    }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
      };
    in
    {
      nixosConfigurations = {
        yourHostname = nixpkgs.lib.nixosSystem {
          inherit system;
          modules = [
            (
              { config, pkgs, ... }:
              {
                environment.systemPackages = [
                  keepass-2-file.packages.${system}.default
                ];
              }
            )
          ];
        };
      };
    };
}