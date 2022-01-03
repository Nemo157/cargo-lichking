# cargo-lichking [![travis-badge][]][travis] [![crate-badge][]][crate] [![license-badge][]][license] [![rust-version-badge][]][rust-version]

Automated **li**cense **ch**ec**king** for rust. `cargo lichking` is a [Cargo][]
subcommand that checks licensing information for dependencies.

**Liches are not lawyers**, the information output from this tool is provided as
a hint to where you may need to look for licensing issues but in no way
represents legal advice or guarantees correctness. The tool relies at a minimum
on package metadata containing correct licensing information, this is not
guaranteed so for real license checking it's necessary to verify all
dependencies manually.

## Rust Version Policy

This crate only supports the current stable version of Rust, patch releases may
use new features at any time.

## Installation

To install simply run `cargo install cargo-lichking`.

## Usage

To get a list of all your (transitive) dependencies licenses run `cargo lichking
list`. To check license compatibility based off this [License Slide][] by David
A. Wheeler run `cargo lichking check`.

## Developing

When running via `cargo run` you'll need to provide an initial `lichking`
argument to simulate running as a cargo subcommand, e.g. `cargo run -- lichking
check`.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.

[travis-badge]: https://img.shields.io/travis/Nemo157/cargo-lichking/master.svg?style=flat-square
[travis]: https://travis-ci.org/Nemo157/cargo-lichking
[crate-badge]: https://img.shields.io/crates/v/cargo-lichking.svg?style=flat-square
[crate]: https://crates.io/crates/cargo-lichking
[license-badge]: https://img.shields.io/crates/l/cargo-lichking.svg?style=flat-square
[license]: #license
[rust-version-badge]: https://img.shields.io/badge/rust-latest%20stable-blue.svg?style=flat-square
[rust-version]: #rust-version-policy

[Cargo]: https://github.com/rust-lang/cargo
[License Slide]: https://dwheeler.com/essays/floss-license-slide.html
