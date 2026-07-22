{pkgs, ...}: {
  env.GREET = "devenv";

  packages = [pkgs.git pkgs.nb pkgs.openssl];

  languages.rust = {
    enable = true;
    channel = "stable";
    version = "latest";
    components = ["rustc" "cargo" "clippy" "rustfmt" "rust-analyzer" "rust-src"];
  };

  enterShell = ''
    rustc --version
    git --version
    reset-nb
  '';

  scripts.reset-nb.exec = ''
    nb notebook rm -f home
    nb add --filename note.md --content "this is a note"
    nb add --filename note.md --content "this is a note"
    nb add --filename note2.md --content "# note with title"
    nb add --filename note3.md --content "# pinned note with title"
    nb pin note3.md
    nb add folder folder
    nb add folder/note.md --content "note inside a folder"
    nb todo add todo
    nb todo add todo2
    nb todo do todo2
    # nb bookmark https://www.google.com
  '';
}
