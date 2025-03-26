{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    utils,
  }:
    utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
    in {
      defaultApp = utils.lib.mkApp {drv = self.defaultPackage."${system}";};

      devShell = with pkgs;
        mkShell rec {
          nativeBuildInputs = [
            pkg-config
          ];
          buildInputs = [
            cargo
            rustc
            rustfmt
            udev
            alsa-lib
            vulkan-loader
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr # To use the x11 feature
            libxkbcommon
            wayland # To use the wayland feature
          ];
          LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
    });
}
