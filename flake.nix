{
  description = "pgn-sort — devShell delegated to chess-flake workspace";

  inputs.workspace.url = "git+ssh://git@github.com/obazin/chess-flake.git?ref=main";

  outputs =
    { self, workspace }:
    {
      devShells = builtins.mapAttrs (system: shells: {
        default = shells.pgn-sort;
      }) workspace.devShells;
    };
}
