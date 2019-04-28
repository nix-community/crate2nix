let nixpkgs = builtins.fetchTarball {
    name = "nixos-unstable-2019-04-28";
    url = https://github.com/nixos/nixpkgs/archive/a2e219ec879fe3de62e9204ad33efb5b67a4a324.tar.gz;
    sha256 = "0bdr8ii6kc1rd4l5dnmqpb5ljh2wk47y75n9ibg2ddc62hi6jxq9";
};

in import nixpkgs
