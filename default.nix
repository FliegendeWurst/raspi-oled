{ lib, fetchFromGitHub, rustPlatform, pkg-config, sqlite }:

rustPlatform.buildRustPackage {
  pname = "raspi-oled";
  version = "unstable-infdev-4";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "ssd1351-0.3.0" = "sha256-K6QCU9qPEuU7Ur8W6fTdi4JWk8NsVz3mLfV0afpfdBA=";
	  # "gpio-am2302-rs-1.1.0" = "sha256-tyA/R80LtWIXoVEoxHhkmzy0IsMdMH1Oi3FTQ56XjyQ=";
    };
  };

  nativeBuildInputs = [ pkg-config ];

  cargoBuildFlags = [ "--no-default-features" ];

  buildInputs = [ sqlite ];

  RUSTC_BOOTSTRAP = "1";

  meta = with lib; {
    description = "OLED display of clock/calendar/temperature";
    homepage = "https://github.com/FliegendeWurst/raspi-oled";
    license = licenses.mit;
    maintainers = with maintainers; [ fliegendewurst ];
  };
}

