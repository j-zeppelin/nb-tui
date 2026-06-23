{pkgs, ...}: {
  env.GREET = "devenv";

  packages = [pkgs.git pkgs.nb];

  languages.rust = {
    enable = true;
    channel = "stable";
    version = "latest";
    components = ["rustc" "cargo" "clippy" "rustfmt" "rust-analyzer" "rust-src"];
  };

  enterShell = ''
    rustc --version
    git --version
  '';
}
