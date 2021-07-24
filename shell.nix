with (import <nixpkgs> {});
mkShell {
  nativeBuildInputs = [
    pkgsCross.muslpi.stdenv.cc
  ];
  CARGO_TARGET_ARM_UNKNOWN_LINUX_MUSLEABIHF_LINKER = "${pkgsCross.muslpi.stdenv.cc}/bin/armv6l-unknown-linux-musleabihf-ld";
}
