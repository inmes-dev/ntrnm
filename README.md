# `ntrnm`

![Rust Version][rustc-image]
[![crates.io][crate-image]][crate-link]
[![Documentation][docs-image]][docs-link]
[![Dependency Status][deps-image]][deps-link]

@TODO: about

<!-- markdown-toc start - Don't edit this section. Run M-x markdown-toc-refresh-toc -->
**Table of Contents**

- [`ntrnm`](#ntrnm)
    - [The Pitch](#the-pitch)
    - [The Anit-Pitch](#the-anit-pitch)
- [Installation](#installation)
    - [Compile from Source](#compile-from-source)
- [Usage](#usage)
    - [Command Line Interface](#command-line-interface)
- [License](#license)
    - [Contribution](#contribution)

<!-- markdown-toc end -->

## The Pitch

@TODO: pitch

## The Anit-Pitch

@TODO: anti-pitch

# Installation

`{{crate_name}}` is a single binary that must be placed somewhere in your
`$PATH`. One can either download 64-bit Linux binaries from [the Release Page](https://github.com/inmes-dev/iptables_exporter/releases)
or one can also compile from source.

## Compile from Source

Ensure you have a [Rust toolchain installed](https://rustup.rs). Some of the
dependencies also require `gcc` to be installed.

```
$ git clone https://github.com/ntqq-group/ntrnm
$ cd ntrnm
$ cargo build --release
$ sudo cp target/release/ntrnm /usr/local/bin/
```

# Usage

## Command Line Interface

```
@TODO: cli usage
```

# License

This crate is licensed under either of

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[//]: # (badges)

[rustc-image]: https://img.shields.io/badge/rustc-1.53+-blue.svg
[crate-image]: https://img.shields.io/crates/v/ntrnm.svg
[crate-link]: https://crates.io/crates/ntrnm
[docs-image]: https://docs.rs/ntrnm/badge.svg
[docs-link]: https://docs.rs/ntrnm
[deps-image]: https://deps.rs/repo/github/inmes-dev/ntrnm/status.svg
[deps-link]: https://deps.rs/repo/github/inmes-dev/ntrnm
