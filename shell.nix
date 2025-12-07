{ pkgs ? import (fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/24.11.tar.gz";
  }) {
    overlays = [
      (import (fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz"))
    ];
  }
}:

let
  # Rust toolchain - use latest stable (should be 1.82+)
  rustToolchain = pkgs.rust-bin.stable.latest.default.override {
    extensions = [ "rust-src" "rust-analyzer" ];
  };

  # macOS specific packages - using newer SDK framework paths
  darwinPackages = pkgs.lib.optionals pkgs.stdenv.isDarwin (
    with pkgs.darwin.apple_sdk.frameworks; [
      Security
      SystemConfiguration
    ] ++ [ pkgs.libiconv ]
  );
in

pkgs.mkShell {
  buildInputs = [
    # Rust toolchain (pinned version)
    rustToolchain

    # Build dependencies
    pkgs.pkg-config
    pkgs.openssl

    # Development tools
    pkgs.gnumake
  ] ++ darwinPackages;

  # Set environment variables for macOS
  shellHook = ''
    ${pkgs.lib.optionalString pkgs.stdenv.isDarwin ''
      export LDFLAGS="-L${pkgs.libiconv}/lib"
      export CPPFLAGS="-I${pkgs.libiconv}/include"
    ''}

    echo "🦀 Payment Engine Development Environment"
    echo "=========================================="
    echo "Platform: ${if pkgs.stdenv.isDarwin then "macOS" else "Linux"}"
    echo "Rust version: $(rustc --version)"
    echo "Cargo version: $(cargo --version)"
    echo ""
    echo "Available commands:"
    echo "  make init       - Initialize development environment"
    echo "  make build      - Build the project"
    echo "  make test       - Run tests"
    echo "  make run-all    - Run with all example inputs"
    echo "  make help       - Show all make targets"
    echo ""
  '';
}
