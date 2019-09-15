let nixpkgs = builtins.fetchTarball {
    name = "nixos-unstable-20190910";
    url = https://github.com/nixos/nixpkgs/archive/a69a6c11176d6aa96ab7c8f7308be4b95206598e.tar.gz;
    sha256 = "0m66fx9xfh91q6dch6q4ir80zr3iga6y902h83rb7swzaqn9gi7m";
};

in import nixpkgs
