{ lib, fetchFromGitHub, rustPlatform, pkg-config, sqlite }:

rustPlatform.buildRustPackage {
  pname = "raspi-oled";
  version = "unstable-infdev-18";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "ssd1351-0.3.0" = "sha256-DD7+NhYwUwD/xC+7ZUNKdhcfsSCOQ9NVEy9lcS47Q5E=";
    };
  };

  nativeBuildInputs = [ pkg-config ];

  cargoBuildFlags = [ "--no-default-features" "--bin" "main_loop" "--bin" "take_measurement" ];

  buildInputs = [ sqlite ];

  env.RUSTC_BOOTSTRAP = "1";

  meta = with lib; {
    description = "OLED display of clock/calendar/temperature";
    homepage = "https://github.com/FliegendeWurst/raspi-oled";
    license = licenses.gpl3;
    maintainers = with maintainers; [ fliegendewurst ];
  };
}

