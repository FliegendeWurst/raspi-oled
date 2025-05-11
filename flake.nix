{
  description = "raspi-oled";

  # Nixpkgs / NixOS version to use.
  inputs.nixpkgs.url = "nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let

      lib = nixpkgs.lib;

      # to work with older version of flakes
      lastModifiedDate = self.lastModifiedDate or self.lastModified or "19700101";

      # Generate a user-friendly version number.
      version = builtins.substring 0 8 lastModifiedDate;

      # System types to support.
      supportedSystems = [
        "x86_64-linux"
        "x86_64-darwin"
        "aarch64-linux"
        "aarch64-darwin"
        "x86_64-linux-cross-muslpi"
      ];

      # Helper function to generate an attrset '{ x86_64-linux = f "x86_64-linux"; ... }'.
      forAllSystems = lib.genAttrs supportedSystems;

      # Nixpkgs instantiated for supported system types.
      nixpkgsFor = forAllSystems (
        system:
        let
          parts = lib.splitString "-cross-" system;
        in
        (
          if (lib.length parts) == 1 then
            import nixpkgs { inherit system; }
          else
            import nixpkgs {
              localSystem = lib.elemAt parts 0;
              hostSystem = lib.elemAt parts 0;
              crossSystem = "armv6l-unknown-linux-musleabihf";
            }
        )
      );

    in
    {

      # Provide some binary packages for selected system types.
      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgsFor.${system};
        in
        {
          raspi-oled = pkgs.rustPlatform.buildRustPackage {
            pname = "raspi-oled";
            version = "0-unstable" + lib.optionalString (self ? version) "-${self.version}";

            src = ./.;
            buildAndTestSubdir = "raspi-oled";

            cargoLock = {
              lockFile = ./Cargo.lock;
              outputHashes = {
                "ssd1351-0.3.0" = "sha256-DD7+NhYwUwD/xC+7ZUNKdhcfsSCOQ9NVEy9lcS47Q5E=";
              };
            };

            nativeBuildInputs = with nixpkgsFor.${lib.elemAt (lib.splitString "-cross-" system) 0}; [
              pkg-config
              rustPlatform.bindgenHook
            ];

            buildInputs = with pkgs; [ sqlite ];

            buildNoDefaultFeatures = true;
            checkNoDefaultFeatures = true;

            env.LIBSQLITE3_SYS_USE_PKG_CONFIG = "1";
            env.RUSTC_BOOTSTRAP = "1";

            meta = with lib; {
              description = "raspi-oled";
              homepage = "https://github.com/FliegendeWurst/raspi-oled";
              license = licenses.gpl3Plus;
              maintainers = with maintainers; [ fliegendewurst ];
              mainProgram = "main_loop";
            };
          };

          music = pkgs.rustPlatform.buildRustPackage {
            pname = "music";
            version = "0-unstable-${toString self.revCount}";

            src = ./.;
            buildAndTestSubdir = "music";

            cargoLock = {
              lockFile = ./Cargo.lock;
              outputHashes = {
                "ssd1351-0.3.0" = "sha256-DD7+NhYwUwD/xC+7ZUNKdhcfsSCOQ9NVEy9lcS47Q5E=";
                "playerctl-rust-wrapper-0.1.0" = "sha256-0sTvZSClSXolh/peIwSno1O0rY7gn7Rj8T37kg4jRD4=";
              };
            };

            nativeBuildInputs = with nixpkgsFor.${lib.elemAt (lib.splitString "-cross-" system) 0}; [
              rustPlatform.bindgenHook
            ];

            buildInputs = with pkgs; [ ];

            buildNoDefaultFeatures = true;
            checkNoDefaultFeatures = true;

            env.RUSTC_BOOTSTRAP = "1";

            meta = with lib; {
              description = "music";
              homepage = "https://github.com/FliegendeWurst/raspi-oled";
              license = licenses.gpl3Plus;
              maintainers = with maintainers; [ fliegendewurst ];
              mainProgram = "music";
            };
          };
        }
      );

      defaultPackage = forAllSystems (system: self.packages.${system}.raspi-oled);
    };
}
