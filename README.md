# cargo-lichking [![travis-badge][]][travis] [![downloads-badge][] ![release-badge][]][crate] [![license-badge][]](#license) [![rust-version-badge][]][rust-version]

Automated **li**cense **ch**ec**king** for rust. `cargo lichking` is a [Cargo][]
subcommand that checks licensing information for dependencies.

**Liches are not lawyers**, the information output from this tool is provided as
a hint to where you may need to look for licensing issues but in no way
represents legal advice or guarantees correctness. The tool relies at a minimum
on package metadata containing correct licensing information, this is not
guaranteed so for real license checking it's necessary to verify all
dependencies manually.

[travis-badge]: https://img.shields.io/travis/Nemo157/cargo-lichking/master.svg?style=flat-square
[downloads-badge]: https://img.shields.io/crates/d/cargo-lichking.svg?style=flat-square
[release-badge]: https://img.shields.io/crates/v/cargo-lichking.svg?style=flat-square
[license-badge]: https://img.shields.io/crates/l/cargo-lichking.svg?style=flat-square
[rust-version-badge]: https://img.shields.io/badge/rust-1.20+-blue.svg?style=flat-square
[travis]: https://travis-ci.org/Nemo157/cargo-lichking
[crate]: https://crates.io/crates/cargo-lichking
[Cargo]: https://github.com/rust-lang/cargo
[rust-version]: .travis.yml#L5

### Installation

To install simply run `cargo install cargo-lichking`; unless you're using a homebrew installed
copy of openssl, then *"simply"* run:

```shall
OPENSSL_ROOT_DIR=`brew --prefix openssl` \
OPENSSL_LIB_DIR=`brew --prefix openssl`/lib \
OPENSSL_INCLUDE_DIR=`brew --prefix openssl`/include \
cargo install cargo-lichking
```

### Usage

To get a list of all your (transitive) dependencies licenses run `cargo lichking
list`. To check license compatibility based off this [License Slide][] by David
A. Wheeler run `cargo lichking check`.

[License Slide]: http://www.dwheeler.com/essays/floss-license-slide.html

## License
[![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2FNemo157%2Fcargo-lichking.svg?type=shield)](https://app.fossa.io/projects/git%2Bgithub.com%2FNemo157%2Fcargo-lichking?ref=badge_shield)


Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.


[![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2FNemo157%2Fcargo-lichking.svg?type=large)](https://app.fossa.io/projects/git%2Bgithub.com%2FNemo157%2Fcargo-lichking?ref=badge_large)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.

## Developing

If building on OS X with a `homebrew` installed copy of OpenSSL you'll need to
specify where this is to enable building `libssh2-sys`.  Use something like:

```sh
OPENSSL_ROOT_DIR=`brew --prefix openssl` \
OPENSSL_LIB_DIR=`brew --prefix openssl`/lib \
OPENSSL_INCLUDE_DIR=`brew --prefix openssl`/include \
cargo build
```

When running via `cargo run` you'll need to provide an initial `lichking`
argument to simulate running as a cargo subcommand, e.g. `cargo run -- lichking
check`.