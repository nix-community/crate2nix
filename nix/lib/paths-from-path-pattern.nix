{ lib }:
rec {
  /**
    Converts a file path with wildcards to a regular expression pattern.
    Handles common wildcards:

    * - matches any number of characters except /
    ** - matches any number of characters including /
    ? - matches exactly one character except /
    {a,b,c} - matches any one of the patterns separated by commas
    [abc] - matches any character in the brackets
    [0-9] - matches any character in the brackets
    [!abc] or [^abc] - matches any character not in the brackets, or in the given range

    Example:
    pathToRegex "crates/*" -> "crates/[^/]*"
  */
  pathToRegex = path:
    let
      # Escape special regex characters except characters used in globs: * ? ^ [ ] { }
      escapeRegex = str:
        lib.replaceStrings
          [ "." "+" "$" "/" "(" ")" "|" "\\" ]
          [ "[.]" "[+]" "[$]" "[/]" "[(]" "[)]" "[|]" "[\\]" ]
          str;
      globToRegex = lib.replaceStrings
        [ "**" "*" "?" "," "{" "}" ]
        [ ".*" "[^/]*" "[^/]" "|" "(" ")" ];
    in
    globToRegex (escapeRegex path);

  /**
    Given a pathPattern that may contain waildcards, find all files and
    directories relative to dir that match the pattern.

    Example:
    ppathsFromPathPattern "crates/*" src -> [ "crates/app" "crates/lib" ]
  */
  pathsFromPathPattern = dir: pathPattern:
    let
      regex = pathToRegex pathPattern;
      helper = subdir: lib.flatten
        (lib.mapAttrsToList
          (name: type:
            let path = if subdir == null then name else subdir + "/${name}"; in
            if builtins.match regex path != null then
              [ path ]
            else if type == "directory" then
              helper path
            else
              [ ]
          )
          (builtins.readDir (if
            subdir == null then
            dir else dir + "/${subdir}"))
        );
    in
    helper null;
}
