let nixpkgs = builtins.fetchTarball {
    name = "nixos-unstable-2019-04-22";
    url = https://github.com/nixos/nixpkgs/archive/d26027792812fbfad4d0f451b5f47fdabf7fdeb9.tar.gz;
    sha256 = "10agvpj4d62yvwxwflgxbg1dxp654r4cwirpbsrmfikp0wsvijsv";
};

in import nixpkgs
