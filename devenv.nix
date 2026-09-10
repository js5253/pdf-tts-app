{ pkgs, ... }: {
  languages = {
    # devenv.sh/languages/rust/
    rust = {
      enable = true;
      channel = "stable";
    };
  };
}