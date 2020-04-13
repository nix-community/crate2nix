let
  # Manage this with https://github.com/nmattia/niv
  # or define { nixpkgs = ...; nixpkgs-mozilla = ...; }
  # yourself.
  sources = import ./sources.nix;

  rustChannelsOverlay = import "${sources.nixpkgs-mozilla}/rust-overlay.nix";
  # Useful if you also want to provide that in a nix-shell since some rust tools depend
  # on that.
  rustChannelsSrcOverlay = import "${sources.nixpkgs-mozilla}/rust-src-overlay.nix";

in import sources.nixpkgs {
    overlays = [
      rustChannelsOverlay
      rustChannelsSrcOverlay
      (self: super: {
        # Replace "latest.rustChannels.stable" with the version of the rust tools that
        # you would like. Look at the documentation of nixpkgs-mozilla for examples.
        #
        # NOTE: "rust" instead of "rustc" is not a typo: It will include more than needed
        # but also the much needed "rust-std".
        rustc = super.latest.rustChannels.nightly.rust;
        inherit (super.latest.rustChannels.nightly) cargo rust rust-fmt rust-std clippy;
      })
    ];
  }
