let
  nixpkgs = fetchTarball "https://github.com/NixOS/nixpkgs/tarball/nixos-25.11";

  pkgs = import nixpkgs {
    config = { };
    overlays = [ ];
  };
in
pkgs.mkShell {
  hardeningDisable = [ "fortify" ];
  buildInputs = with pkgs; [
    pkg-config
    openssl
    rust-analyzer
    cargo-audit
    gcc
    gnumake
    goreleaser
  ];
}
