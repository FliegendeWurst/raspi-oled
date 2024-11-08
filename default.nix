{ lib, fetchFromGitHub, rustPlatform, pkg-config, sqlite }:

rustPlatform.buildRustPackage {
  pname = "raspi-oled";
  version = "unstable-infdev-6";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "ssd1351-0.3.0" = "sha256-DD7+NhYwUwD/xC+7ZUNKdhcfsSCOQ9NVEy9lcS47Q5E=";
	  # "gpio-am2302-rs-1.1.0" = "sha256-tyA/R80LtWIXoVEoxHhkmzy0IsMdMH1Oi3FTQ56XjyQ=";
    };
  };

  nativeBuildInputs = [ pkg-config ];

  cargoBuildFlags = [ "--no-default-features" "--bin" "main_loop" ];

  buildInputs = [ sqlite ];

  RUSTC_BOOTSTRAP = "1";

  meta = with lib; {
    description = "OLED display of clock/calendar/temperature";
    homepage = "https://github.com/FliegendeWurst/raspi-oled";
    license = licenses.gpl3;
    maintainers = with maintainers; [ fliegendewurst ];
  };
}

