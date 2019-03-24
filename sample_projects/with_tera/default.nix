{ pkgs? import <nixos-unstable> { config = {}; },
  lib? pkgs.lib,
  callPackage? pkgs.callPackage,
  stdenv? pkgs.stdenv,
  buildRustCrate? pkgs.buildRustCrate,
  fetchurl? pkgs.fetchurl }:

rec {
    root_crate = crates."tera 1.0.0-beta.2 (registry+https://github.com/rust-lang/crates.io-index)";
    crates = {
        "aho-corasick 0.6.10 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "aho-corasick";
                version = "0.6.10";
                edition = "2015";
                # Hack to suppress building binaries
                crateBin = [{name = ","; path = ",";}];
                sha256 = "0bhasxfpmfmz1460chwsx59vdld05axvmk1nbp3sd48xav3d108p";
                libPath = "src/lib.rs";
                authors = [
                    "Andrew Gallant <jamslam@gmail.com>"
                ];
                dependencies = [
                    crates."memchr 2.2.0 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "arrayref 0.3.5 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "arrayref";
                version = "0.3.5";
                edition = "2015";
                sha256 = "00dfn9lbr4pc524imc25v3rbmswiqk3jldsgmx4rdngcpxb8ssjf";
                libPath = "src/lib.rs";
                authors = [
                    "David Roundy <roundyd@physics.oregonstate.edu>"
                ];
            };
        "block-buffer 0.3.3 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "block-buffer";
                version = "0.3.3";
                edition = "2015";
                sha256 = "0ka14535hlndyig1dqxqvdv60mgmnnhfi6x87npha3x3yg5sx201";
                libPath = "src/lib.rs";
                authors = [
                    "RustCrypto Developers"
                ];
                dependencies = [
                    crates."arrayref 0.3.5 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."byte-tools 0.2.0 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "byte-tools 0.2.0 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "byte-tools";
                version = "0.2.0";
                edition = "2015";
                sha256 = "15cm6sxkk2ikrz8sxld3hv9g419j4kjzwdjp4fn53gjq07awq6il";
                libPath = "src/lib.rs";
                authors = [
                    "The Rust-Crypto Project Developers"
                ];
            };
        "cfg-if 0.1.6 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "cfg-if";
                version = "0.1.6";
                edition = "2015";
                sha256 = "11qrix06wagkplyk908i3423ps9m9np6c4vbcq81s9fyl244xv3n";
                libPath = "src/lib.rs";
                authors = [
                    "Alex Crichton <alex@alexcrichton.com>"
                ];
            };
        "chrono 0.4.6 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "chrono";
                version = "0.4.6";
                edition = "2015";
                sha256 = "0cxgqgf4lknsii1k806dpmzapi2zccjpa350ns5wpb568mij096x";
                libPath = "src/lib.rs";
                authors = [
                    "Kang Seonghoon <public+rust@mearie.org>"
                    "Brandon W Maister <quodlibetor@gmail.com>"
                ];
                dependencies = [
                    crates."time 0.1.42 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."num-traits 0.2.6 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."num-integer 0.1.39 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
                features = [
                   "clock"
                   "default"
                   "time"
                ];
            };
        "crossbeam-channel 0.3.8 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "crossbeam-channel";
                version = "0.3.8";
                edition = "2015";
                sha256 = "0apm8why2qsgr8ykh9x677kc9ml7qp71mvirfkdzdn4c1jyqyyzm";
                libPath = "src/lib.rs";
                authors = [
                    "The Crossbeam Project Developers"
                ];
                dependencies = [
                    crates."smallvec 0.6.9 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."crossbeam-utils 0.6.5 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "crossbeam-utils 0.6.5 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "crossbeam-utils";
                version = "0.6.5";
                edition = "2015";
                sha256 = "1z7wgcl9d22r2x6769r5945rnwf3jqfrrmb16q7kzk292r1d4rdg";
                libPath = "src/lib.rs";
                authors = [
                    "The Crossbeam Project Developers"
                ];
                dependencies = [
                    crates."lazy_static 1.3.0 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."cfg-if 0.1.6 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
                features = [
                   "default"
                   "lazy_static"
                   "std"
                ];
            };
        "deunicode 0.4.3 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "deunicode";
                version = "0.4.3";
                edition = "2015";
                sha256 = "138gv1cadzf9pf6i2wnmsbk927zfivpp0g74c1jnp7z3g4mwfdgl";
                libPath = "src/lib.rs";
                authors = [
                    "Kornel Lesinski <kornel@geekhood.net>"
                    "Amit Chowdhury <amitc97@gmail.com>"
                ];
            };
        "digest 0.7.6 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "digest";
                version = "0.7.6";
                edition = "2015";
                sha256 = "074cw6sk5qfha3gjwgx3fg50z64wrabszfkrda2mi6b3rjrk80d4";
                libPath = "src/lib.rs";
                authors = [
                    "RustCrypto Developers"
                ];
                dependencies = [
                    crates."generic-array 0.9.0 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "fake-simd 0.1.2 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "fake-simd";
                version = "0.1.2";
                edition = "2015";
                sha256 = "1a0f1j66nkwfy17s06vm2bn9vh8vy8llcijfhh9m10p58v08661a";
                libPath = "src/lib.rs";
                authors = [
                    "The Rust-Crypto Project Developers"
                ];
            };
        "fnv 1.0.6 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "fnv";
                version = "1.0.6";
                edition = "2015";
                sha256 = "128mlh23y3gg6ag5h8iiqlcbl59smisdzraqy88ldrf75kbw27ip";
                libPath = "lib.rs";
                authors = [
                    "Alex Crichton <alex@alexcrichton.com>"
                ];
            };
        "generic-array 0.9.0 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "generic-array";
                version = "0.9.0";
                edition = "2015";
                sha256 = "1gk3g5yxvh361syfz38nlf6vg7d0qx7crpa83mnqzaf9dymz19g7";
                libPath = "src/lib.rs";
                authors = [
                    "Bartłomiej Kamiński <fizyk20@gmail.com>"
                ];
                dependencies = [
                    crates."typenum 1.10.0 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "globset 0.4.2 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "globset";
                version = "0.4.2";
                edition = "2015";
                sha256 = "0cymxnzzzadk13f344gska1apvggc0mnd3klhw3h504vhqrb1l2b";
                libPath = "src/lib.rs";
                authors = [
                    "Andrew Gallant <jamslam@gmail.com>"
                ];
                dependencies = [
                    crates."memchr 2.2.0 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."aho-corasick 0.6.10 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."log 0.4.6 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."fnv 1.0.6 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."regex 1.1.2 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "globwalk 0.7.0 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "globwalk";
                version = "0.7.0";
                edition = "2015";
                sha256 = "0i9d51hb06m4pq1mnk3x2rxi60283yhjw6my3i0nhpjsf97hrqfj";
                libPath = "src/lib.rs";
                authors = [
                    "Gilad Naaman <gilad@naaman.io>"
                ];
                dependencies = [
                    crates."walkdir 2.2.7 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."ignore 0.4.6 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "humansize 1.1.0 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "humansize";
                version = "1.1.0";
                edition = "2015";
                sha256 = "1s7jj570vz90b7wsgd24lla1yn9qp3swgv9c7jgkgrw6bxynbv0p";
                libPath = "src/lib.rs";
                authors = [
                    "Leopold Arkham <leopold.arkham@gmail.com>"
                ];
            };
        "idna 0.1.5 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "idna";
                version = "0.1.5";
                edition = "2015";
                sha256 = "1gwgl19rz5vzi67rrhamczhxy050f5ynx4ybabfapyalv7z1qmjy";
                libPath = "src/lib.rs";
                authors = [
                    "The rust-url developers"
                ];
                dependencies = [
                    crates."unicode-normalization 0.1.8 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."matches 0.1.8 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."unicode-bidi 0.3.4 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "ignore 0.4.6 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "ignore";
                version = "0.4.6";
                edition = "2015";
                sha256 = "1gx1dia1ws3qm2m7pxfnsp43i0wz2fkzn4yv6zxqzib7qp3cpzb6";
                libPath = "src/lib.rs";
                authors = [
                    "Andrew Gallant <jamslam@gmail.com>"
                ];
                dependencies = [
                    crates."regex 1.1.2 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."thread_local 0.3.6 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."globset 0.4.2 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."crossbeam-channel 0.3.8 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."walkdir 2.2.7 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."memchr 2.2.0 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."lazy_static 1.3.0 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."same-file 1.0.4 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."log 0.4.6 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "itoa 0.4.3 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "itoa";
                version = "0.4.3";
                edition = "2015";
                sha256 = "0zadimmdgvili3gdwxqg7ljv3r4wcdg1kkdfp9nl15vnm23vrhy1";
                libPath = "src/lib.rs";
                authors = [
                    "David Tolnay <dtolnay@gmail.com>"
                ];
                features = [
                   "default"
                   "std"
                ];
            };
        "lazy_static 1.3.0 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "lazy_static";
                version = "1.3.0";
                edition = "2015";
                sha256 = "1vv47va18ydk7dx5paz88g3jy1d3lwbx6qpxkbj8gyfv770i4b1y";
                libPath = "src/lib.rs";
                authors = [
                    "Marvin Löbel <loebel.marvin@gmail.com>"
                ];
            };
        "libc 0.2.49 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "libc";
                version = "0.2.49";
                edition = "2015";
                sha256 = "0idjzk8zq106a50mx5ksi6c2ghd0949fb1r4rq8ygak3ilz76x74";
                build = "build.rs";
                libPath = "src/lib.rs";
                authors = [
                    "The Rust Project Developers"
                ];
                features = [
                   "default"
                   "use_std"
                ];
            };
        "log 0.4.6 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "log";
                version = "0.4.6";
                edition = "2015";
                sha256 = "1nd8dl9mvc9vd6fks5d4gsxaz990xi6rzlb8ymllshmwi153vngr";
                libPath = "src/lib.rs";
                authors = [
                    "The Rust Project Developers"
                ];
                dependencies = [
                    crates."cfg-if 0.1.6 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "maplit 1.0.1 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "maplit";
                version = "1.0.1";
                edition = "2015";
                sha256 = "1lcadhrcy2qyb6zazmzj7gvgb50rmlvkcivw287016j4q723x72g";
                libPath = "src/lib.rs";
                authors = [
                    "bluss"
                ];
            };
        "matches 0.1.8 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "matches";
                version = "0.1.8";
                edition = "2015";
                sha256 = "03hl636fg6xggy0a26200xs74amk3k9n0908rga2szn68agyz3cv";
                libPath = "lib.rs";
                authors = [
                    "Simon Sapin <simon.sapin@exyr.org>"
                ];
            };
        "memchr 2.2.0 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "memchr";
                version = "2.2.0";
                edition = "2015";
                sha256 = "11vwg8iig9jyjxq3n1cq15g29ikzw5l7ar87md54k1aisjs0997p";
                build = "build.rs";
                libPath = "src/lib.rs";
                authors = [
                    "Andrew Gallant <jamslam@gmail.com>"
                    "bluss"
                ];
                features = [
                   "default"
                   "use_std"
                ];
            };
        "nom 4.2.1 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "nom";
                version = "4.2.1";
                edition = "2015";
                sha256 = "17cw4aybhaifgyj2v813hwrky5zc55jnknfsbkpmd8clsxkiqclf";
                libPath = "src/lib.rs";
                authors = [
                    "contact@geoffroycouprie.com"
                ];
                dependencies = [
                    crates."memchr 2.2.0 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
                buildDependencies = [
                    crates."version_check 0.1.5 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
                features = [
                   "alloc"
                   "default"
                   "memchr"
                   "std"
                ];
            };
        "num-integer 0.1.39 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "num-integer";
                version = "0.1.39";
                edition = "2015";
                sha256 = "1f42ls46cghs13qfzgbd7syib2zc6m7hlmv1qlar6c9mdxapvvbg";
                build = "build.rs";
                libPath = "src/lib.rs";
                authors = [
                    "The Rust Project Developers"
                ];
                dependencies = [
                    crates."num-traits 0.2.6 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "num-traits 0.2.6 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "num-traits";
                version = "0.2.6";
                edition = "2015";
                sha256 = "1d20sil9n0wgznd1nycm3yjfj1mzyl41ambb7by1apxlyiil1azk";
                build = "build.rs";
                libPath = "src/lib.rs";
                authors = [
                    "The Rust Project Developers"
                ];
            };
        "percent-encoding 1.0.1 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "percent-encoding";
                version = "1.0.1";
                edition = "2015";
                sha256 = "04ahrp7aw4ip7fmadb0bknybmkfav0kk0gw4ps3ydq5w6hr0ib5i";
                libPath = "lib.rs";
                authors = [
                    "The rust-url developers"
                ];
            };
        "pest 2.1.0 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "pest";
                version = "2.1.0";
                edition = "2015";
                sha256 = "1kh0j52na3pm49zqnqw85q2y8wbim0sgza5j861qdmbbp4rdz1xc";
                libPath = "src/lib.rs";
                authors = [
                    "Dragoș Tiselice <dragostiselice@gmail.com>"
                ];
                dependencies = [
                    crates."ucd-trie 0.1.1 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "pest_derive 2.1.0 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "pest_derive";
                version = "2.1.0";
                edition = "2015";
                sha256 = "03bsaw7jpsk6x3dbrs9bjx5najjdvslb9y77azfn1n403khrqvnm";
                procMacro = true;
                authors = [
                    "Dragoș Tiselice <dragostiselice@gmail.com>"
                ];
                dependencies = [
                    crates."pest_generator 2.1.0 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."pest 2.1.0 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "pest_generator 2.1.0 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "pest_generator";
                version = "2.1.0";
                edition = "2015";
                sha256 = "089vf75na5md5jip3gl6m6k97x7qq195jxqvgxi8pa6vcszy4a50";
                libPath = "src/lib.rs";
                authors = [
                    "Dragoș Tiselice <dragostiselice@gmail.com>"
                ];
                dependencies = [
                    crates."pest_meta 2.1.0 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."pest 2.1.0 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."syn 0.15.27 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."proc-macro2 0.4.27 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."quote 0.6.11 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "pest_meta 2.1.0 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "pest_meta";
                version = "2.1.0";
                edition = "2015";
                sha256 = "0f07nxzdqc5hca9m9wwqfy7mq68nya48k6lra3c26w9hr85lyyr3";
                libPath = "src/lib.rs";
                authors = [
                    "Dragoș Tiselice <dragostiselice@gmail.com>"
                ];
                dependencies = [
                    crates."maplit 1.0.1 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."pest 2.1.0 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
                buildDependencies = [
                    crates."sha-1 0.7.0 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "proc-macro2 0.4.27 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "proc-macro2";
                version = "0.4.27";
                edition = "2015";
                sha256 = "1cp4c40p3hwn2sz72ssqa62gp5n8w4gbamdqvvadzp5l7gxnq95i";
                build = "build.rs";
                libPath = "src/lib.rs";
                authors = [
                    "Alex Crichton <alex@alexcrichton.com>"
                ];
                dependencies = [
                    crates."unicode-xid 0.1.0 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
                features = [
                   "default"
                   "proc-macro"
                ];
            };
        "quote 0.6.11 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "quote";
                version = "0.6.11";
                edition = "2015";
                sha256 = "0agska77z58cypcq4knayzwx7r7n6m756z1cz9cp2z4sv0b846ga";
                libPath = "src/lib.rs";
                authors = [
                    "David Tolnay <dtolnay@gmail.com>"
                ];
                dependencies = [
                    crates."proc-macro2 0.4.27 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
                features = [
                   "default"
                   "proc-macro"
                   "proc-macro2"
                ];
            };
        "redox_syscall 0.1.51 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "redox_syscall";
                version = "0.1.51";
                edition = "2015";
                sha256 = "1a61cv7yydx64vpyvzr0z0hwzdvy4gcvcnfc6k70zpkngj5sz3ip";
                libPath = "src/lib.rs";
                authors = [
                    "Jeremy Soller <jackpot51@gmail.com>"
                ];
            };
        "regex 1.1.2 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "regex";
                version = "1.1.2";
                edition = "2015";
                sha256 = "1c9nb031z1vw5l6lzfkfra2mah9hb2s1wgq9f1lmgcbkiiprj9xd";
                build = "build.rs";
                libPath = "src/lib.rs";
                authors = [
                    "The Rust Project Developers"
                ];
                dependencies = [
                    crates."aho-corasick 0.6.10 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."thread_local 0.3.6 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."utf8-ranges 1.0.2 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."regex-syntax 0.6.5 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."memchr 2.2.0 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
                features = [
                   "default"
                   "use_std"
                ];
            };
        "regex-syntax 0.6.5 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "regex-syntax";
                version = "0.6.5";
                edition = "2015";
                sha256 = "0aaaba1fan2qfyc31wzdmgmbmyirc27zgcbz41ba5wm1lb2d8kli";
                libPath = "src/lib.rs";
                authors = [
                    "The Rust Project Developers"
                ];
                dependencies = [
                    crates."ucd-util 0.1.3 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "ryu 0.2.7 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "ryu";
                version = "0.2.7";
                edition = "2015";
                sha256 = "0m8szf1m87wfqkwh1f9zp9bn2mb0m9nav028xxnd0hlig90b44bd";
                build = "build.rs";
                libPath = "src/lib.rs";
                authors = [
                    "David Tolnay <dtolnay@gmail.com>"
                ];
            };
        "same-file 1.0.4 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "same-file";
                version = "1.0.4";
                edition = "2015";
                sha256 = "1zs244ssl381cqlnh2g42g3i60qip4z72i26z44d6kas3y3gy77q";
                libPath = "src/lib.rs";
                authors = [
                    "Andrew Gallant <jamslam@gmail.com>"
                ];
            };
        "serde 1.0.89 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "serde";
                version = "1.0.89";
                edition = "2015";
                sha256 = "14pidc6skkm92vhp431wi1aam5vv5g6rmsimik38wzb0qy72c71g";
                build = "build.rs";
                libPath = "src/lib.rs";
                authors = [
                    "Erick Tryzelaar <erick.tryzelaar@gmail.com>"
                    "David Tolnay <dtolnay@gmail.com>"
                ];
                features = [
                   "default"
                   "std"
                ];
            };
        "serde_json 1.0.39 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "serde_json";
                version = "1.0.39";
                edition = "2015";
                sha256 = "07ydv06hn8x0yl0rc94l2wl9r2xz1fqd97n1s6j3bgdc6gw406a8";
                libPath = "src/lib.rs";
                authors = [
                    "Erick Tryzelaar <erick.tryzelaar@gmail.com>"
                    "David Tolnay <dtolnay@gmail.com>"
                ];
                dependencies = [
                    crates."ryu 0.2.7 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."serde 1.0.89 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."itoa 0.4.3 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
                features = [
                   "default"
                ];
            };
        "sha-1 0.7.0 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "sha-1";
                version = "0.7.0";
                edition = "2015";
                sha256 = "0dxvdr91zc6wi8wgbs0h745b6315xxdy2zpn4qmxnaqzzc55dpxz";
                libPath = "src/lib.rs";
                authors = [
                    "RustCrypto Developers"
                ];
                dependencies = [
                    crates."byte-tools 0.2.0 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."block-buffer 0.3.3 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."digest 0.7.6 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."fake-simd 0.1.2 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "slug 0.1.4 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "slug";
                version = "0.1.4";
                edition = "2015";
                sha256 = "0ijcaqi7mxbib6jag9ldjph9kd3ajh13rajqmzpvd3nqn5mdk6p4";
                libPath = "src/lib.rs";
                authors = [
                    "Steven Allen <steven@stebalien.com>"
                ];
                dependencies = [
                    crates."deunicode 0.4.3 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "smallvec 0.6.9 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "smallvec";
                version = "0.6.9";
                edition = "2015";
                sha256 = "0p96l51a2pq5y0vn48nhbm6qslbc6k8h28cxm0pmzkqmj7xynz6w";
                libPath = "lib.rs";
                authors = [
                    "Simon Sapin <simon.sapin@exyr.org>"
                ];
                features = [
                   "default"
                   "std"
                ];
            };
        "syn 0.15.27 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "syn";
                version = "0.15.27";
                edition = "2015";
                sha256 = "0xc1pi87r8f1j2i1mqaf31nx8417bhdir6xqdxrv03gvhr2drk64";
                libPath = "src/lib.rs";
                authors = [
                    "David Tolnay <dtolnay@gmail.com>"
                ];
                dependencies = [
                    crates."unicode-xid 0.1.0 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."proc-macro2 0.4.27 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."quote 0.6.11 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
                features = [
                   "clone-impls"
                   "default"
                   "derive"
                   "parsing"
                   "printing"
                   "proc-macro"
                   "proc-macro2"
                   "quote"
                ];
            };
        "tera 1.0.0-beta.2 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "tera";
                version = "1.0.0-beta.2";
                edition = "2018";
                sha256 = "0k75cakcxkwk0wbbxrx10dcch4qphn6i7jd60fbdrfc8vrlwwa36";
                libPath = "src/lib.rs";
                authors = [
                    "Vincent Prouillet <prouillet.vincent@gmail.com>"
                ];
                dependencies = [
                    crates."v_htmlescape 0.4.2 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."lazy_static 1.3.0 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."url 1.7.2 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."serde 1.0.89 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."unic-segment 0.8.0 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."humansize 1.1.0 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."chrono 0.4.6 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."pest_derive 2.1.0 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."pest 2.1.0 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."slug 0.1.4 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."globwalk 0.7.0 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."serde_json 1.0.39 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."regex 1.1.2 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
                features = [
                   "default"
                ];
            };
        "thread_local 0.3.6 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "thread_local";
                version = "0.3.6";
                edition = "2015";
                sha256 = "02rksdwjmz2pw9bmgbb4c0bgkbq5z6nvg510sq1s6y2j1gam0c7i";
                libPath = "src/lib.rs";
                authors = [
                    "Amanieu d'Antras <amanieu@gmail.com>"
                ];
                dependencies = [
                    crates."lazy_static 1.3.0 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "time 0.1.42 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "time";
                version = "0.1.42";
                edition = "2015";
                sha256 = "1ny809kmdjwd4b478ipc33dz7q6nq7rxk766x8cnrg6zygcksmmx";
                libPath = "src/lib.rs";
                authors = [
                    "The Rust Project Developers"
                ];
                dependencies = [
                    crates."libc 0.2.49 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "typenum 1.10.0 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "typenum";
                version = "1.10.0";
                edition = "2015";
                sha256 = "1v2cgg0mlzkg5prs7swysckgk2ay6bpda8m83c2sn3z77dcsx3bc";
                build = "build/main.rs";
                libPath = "src/lib.rs";
                authors = [
                    "Paho Lurie-Gregg <paho@paholg.com>"
                    "Andre Bogus <bogusandre@gmail.com>"
                ];
            };
        "ucd-trie 0.1.1 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "ucd-trie";
                version = "0.1.1";
                edition = "2015";
                sha256 = "12awmddm5d31whgd1q309gz1q8qa4i1h9fwf1h71k1d4ivxss68m";
                libPath = "src/lib.rs";
                authors = [
                    "Andrew Gallant <jamslam@gmail.com>"
                ];
                features = [
                   "default"
                   "std"
                ];
            };
        "ucd-util 0.1.3 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "ucd-util";
                version = "0.1.3";
                edition = "2015";
                sha256 = "1n1qi3jywq5syq90z9qd8qzbn58pcjgv1sx4sdmipm4jf9zanz15";
                libPath = "src/lib.rs";
                authors = [
                    "Andrew Gallant <jamslam@gmail.com>"
                ];
            };
        "unic-char-property 0.8.0 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "unic-char-property";
                version = "0.8.0";
                edition = "2015";
                sha256 = "028hsp9lcd41j54nnpqa9gmvjbwai8qw3n7p0cyzf0zwcma650vd";
                libPath = "src/lib.rs";
                authors = [
                    "The UNIC Project Developers"
                ];
                dependencies = [
                    crates."unic-char-range 0.8.0 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "unic-char-range 0.8.0 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "unic-char-range";
                version = "0.8.0";
                edition = "2015";
                sha256 = "1chvp4jj00yhz4mn9fw6v2p1kmfviq6jbyw4fgqfcp5ra1792wvf";
                libPath = "src/lib.rs";
                authors = [
                    "The UNIC Project Developers"
                ];
                features = [
                   "default"
                ];
            };
        "unic-common 0.8.0 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "unic-common";
                version = "0.8.0";
                edition = "2015";
                sha256 = "052vq7r17klj4d3anjv8lygyyh5v67rxmh3jvbphjshmc3hxd5i4";
                libPath = "src/lib.rs";
                authors = [
                    "The UNIC Project Developers"
                ];
                features = [
                   "default"
                ];
            };
        "unic-segment 0.8.0 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "unic-segment";
                version = "0.8.0";
                edition = "2015";
                sha256 = "1wvdiib14zivaqla9wwvq133yqd39yxqf0d0zb6gq6z1xkq0lcp9";
                libPath = "src/lib.rs";
                authors = [
                    "The UNIC Project Developers"
                ];
                dependencies = [
                    crates."unic-ucd-segment 0.8.0 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "unic-ucd-segment 0.8.0 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "unic-ucd-segment";
                version = "0.8.0";
                edition = "2015";
                sha256 = "14ls4fx4rdrrhh7zbmshhgikcwcs8nzzw613mwpccyn4pnhm7qxj";
                libPath = "src/lib.rs";
                authors = [
                    "The UNIC Project Developers"
                ];
                dependencies = [
                    crates."unic-char-property 0.8.0 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."unic-ucd-version 0.8.0 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."unic-char-range 0.8.0 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "unic-ucd-version 0.8.0 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "unic-ucd-version";
                version = "0.8.0";
                edition = "2015";
                sha256 = "0waab5maacd4b25xgp7mhbasw2kg9szg0lachygp3f5nw7wjnf77";
                libPath = "src/lib.rs";
                authors = [
                    "The UNIC Project Developers"
                ];
                dependencies = [
                    crates."unic-common 0.8.0 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "unicode-bidi 0.3.4 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "unicode-bidi";
                version = "0.3.4";
                edition = "2015";
                sha256 = "0lcd6jasrf8p9p0q20qyf10c6xhvw40m2c4rr105hbk6zy26nj1q";
                libPath = "src/lib.rs";
                authors = [
                    "The Servo Project Developers"
                ];
                dependencies = [
                    crates."matches 0.1.8 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
                features = [
                   "default"
                ];
            };
        "unicode-normalization 0.1.8 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "unicode-normalization";
                version = "0.1.8";
                edition = "2015";
                sha256 = "1pb26i2xd5zz0icabyqahikpca0iwj2jd4145pczc4bb7p641dsz";
                libPath = "src/lib.rs";
                authors = [
                    "kwantam <kwantam@gmail.com>"
                ];
                dependencies = [
                    crates."smallvec 0.6.9 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "unicode-xid 0.1.0 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "unicode-xid";
                version = "0.1.0";
                edition = "2015";
                sha256 = "05wdmwlfzxhq3nhsxn6wx4q8dhxzzfb9szsz6wiw092m1rjj01zj";
                libPath = "src/lib.rs";
                authors = [
                    "erick.tryzelaar <erick.tryzelaar@gmail.com>"
                    "kwantam <kwantam@gmail.com>"
                ];
                features = [
                   "default"
                ];
            };
        "url 1.7.2 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "url";
                version = "1.7.2";
                edition = "2015";
                sha256 = "0qzrjzd9r1niv7037x4cgnv98fs1vj0k18lpxx890ipc47x5gc09";
                libPath = "src/lib.rs";
                authors = [
                    "The rust-url developers"
                ];
                dependencies = [
                    crates."matches 0.1.8 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."percent-encoding 1.0.1 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."idna 0.1.5 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "utf8-ranges 1.0.2 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "utf8-ranges";
                version = "1.0.2";
                edition = "2015";
                sha256 = "1my02laqsgnd8ib4dvjgd4rilprqjad6pb9jj9vi67csi5qs2281";
                libPath = "src/lib.rs";
                authors = [
                    "Andrew Gallant <jamslam@gmail.com>"
                ];
            };
        "v_escape 0.7.1 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "v_escape";
                version = "0.7.1";
                edition = "2018";
                sha256 = "1v47w9524pi9msa11cikwpnkq73zc34wqkn6lxhzh92m70fvwch1";
                build = "build.rs";
                libPath = "src/lib.rs";
                authors = [
                    "Rust-iendo Barcelona <riendocontributions@gmail.com>"
                ];
                dependencies = [
                    crates."version_check 0.1.5 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."v_escape_derive 0.5.1 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
                buildDependencies = [
                    crates."version_check 0.1.5 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "v_escape_derive 0.5.1 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "v_escape_derive";
                version = "0.5.1";
                edition = "2018";
                sha256 = "1r0xvy6iiq1rx4jkkg7ynl1y7s007yh24v1h07mw1apw68lj89sv";
                procMacro = true;
                authors = [
                    "Rust-iendo Barcelona <riendocontributions@gmail.com>"
                ];
                dependencies = [
                    crates."nom 4.2.1 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."quote 0.6.11 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."syn 0.15.27 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."proc-macro2 0.4.27 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "v_htmlescape 0.4.2 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "v_htmlescape";
                version = "0.4.2";
                edition = "2018";
                sha256 = "1ml00xvg3fz04lk6n44v135gypbqwfbm2zk74g9b7rh8xb948xsr";
                build = "build.rs";
                libPath = "src/lib.rs";
                authors = [
                    "Rust-iendo Barcelona <riendocontributions@gmail.com>"
                ];
                dependencies = [
                    crates."cfg-if 0.1.6 (registry+https://github.com/rust-lang/crates.io-index)"
                    crates."v_escape 0.7.1 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
                buildDependencies = [
                    crates."v_escape 0.7.1 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "version_check 0.1.5 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "version_check";
                version = "0.1.5";
                edition = "2015";
                sha256 = "1yrx9xblmwbafw2firxyqbj8f771kkzfd24n3q7xgwiqyhi0y8qd";
                libPath = "src/lib.rs";
                authors = [
                    "Sergio Benitez <sb@sergio.bz>"
                ];
            };
        "walkdir 2.2.7 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "walkdir";
                version = "2.2.7";
                edition = "2015";
                sha256 = "0wq3v28916kkla29yyi0g0xfc16apwx24py68049kriz3gjlig03";
                libPath = "src/lib.rs";
                authors = [
                    "Andrew Gallant <jamslam@gmail.com>"
                ];
                dependencies = [
                    crates."same-file 1.0.4 (registry+https://github.com/rust-lang/crates.io-index)"
                ];
            };
        "winapi 0.3.6 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "winapi";
                version = "0.3.6";
                edition = "2015";
                sha256 = "1d9jfp4cjd82sr1q4dgdlrkvm33zhhav9d7ihr0nivqbncr059m4";
                build = "build.rs";
                libPath = "src/lib.rs";
                authors = [
                    "Peter Atashian <retep998@gmail.com>"
                ];
                features = [
                   "consoleapi"
                   "errhandlingapi"
                   "fileapi"
                   "minwinbase"
                   "minwindef"
                   "ntdef"
                   "processenv"
                   "profileapi"
                   "std"
                   "sysinfoapi"
                   "timezoneapi"
                   "winbase"
                   "wincon"
                   "winerror"
                   "winnt"
                ];
            };
        "winapi-i686-pc-windows-gnu 0.4.0 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "winapi-i686-pc-windows-gnu";
                version = "0.4.0";
                edition = "2015";
                sha256 = "05ihkij18r4gamjpxj4gra24514can762imjzlmak5wlzidplzrp";
                build = "build.rs";
                libPath = "src/lib.rs";
                authors = [
                    "Peter Atashian <retep998@gmail.com>"
                ];
            };
        "winapi-util 0.1.2 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "winapi-util";
                version = "0.1.2";
                edition = "2015";
                sha256 = "07jj7rg7nndd7bqhjin1xphbv8kb5clvhzpqpxkvm3wl84r3mj1h";
                libPath = "src/lib.rs";
                authors = [
                    "Andrew Gallant <jamslam@gmail.com>"
                ];
            };
        "winapi-x86_64-pc-windows-gnu 0.4.0 (registry+https://github.com/rust-lang/crates.io-index)"
            = buildRustCrate {
                crateName = "winapi-x86_64-pc-windows-gnu";
                version = "0.4.0";
                edition = "2015";
                sha256 = "0n1ylmlsb8yg1v583i4xy0qmqg42275flvbc51hdqjjfjcl9vlbj";
                build = "build.rs";
                libPath = "src/lib.rs";
                authors = [
                    "Peter Atashian <retep998@gmail.com>"
                ];
            };
    };

    # Utility functions

    # sourceFilter: Filters common temp files and build files
    # TODO(pkolloch): Substitute with gitignore filter
    sourceFilter = name: type:
        let baseName = builtins.baseNameOf (builtins.toString name);
        in ! (
          # Filter out git
          baseName == ".gitignore" ||
          (type == "directory" && baseName == ".git" ) ||

          # Filter out build results
          (type == "directory" && (
            baseName == "target" ||
            baseName == "_site" ||
            baseName == ".sass-cache" ||
            baseName == ".jekyll-metadata" ||
            baseName == "build-artifacts"
            )) ||

          # Filter out nix-build result symlinks
          (type == "symlink" && lib.hasPrefix "result" baseName) ||

          # Filter out IDE config
          (type == "directory" && (
            baseName == ".idea" ||
            baseName == ".vscode"
            )) ||
          lib.hasSuffix ".iml" baseName ||

          # Filter out nix build files
          lib.hasSuffix ".nix" baseName ||

          # Filter out editor backup / swap files.
          lib.hasSuffix "~" baseName ||
          builtins.match "^\\.sw[a-z]$" baseName != null ||
          builtins.match "^\\..*\\.sw[a-z]$" baseName != null ||
          lib.hasSuffix ".tmp" baseName ||
          lib.hasSuffix ".bak" baseName
        );

}

