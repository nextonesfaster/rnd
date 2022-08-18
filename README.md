# rnd

`rnd` is a simple command-line tool that lets you select random data in different ways.

I made this for my personal use, but you may find it useful if you need to choose items or shuffle lists often.

## Usage

```txt
rnd 0.1.0
Sujal Bolia <sujalbolia@gmail.com>

rnd lets you select random data in different ways

USAGE:
    rnd <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    choose     Choose `amount` elements from a list of items
    coin       Flip a coin `amount` times
    help       Print this message or the help of the given subcommand(s)
    random     Print a random number between 0.0 and 1.0 (not inclusive)
    shuffle    Shuffle a list of items
```

## Installation

You need [Rust][rust] to compile `rnd`. Pre-compiled binaries are not available yet.

`cargo` is usually installed with Rust. If you don't have `cargo` installed, follow [the `cargo` installation documentation][cargo].

Once you have `cargo` installed, you can simply use `cargo install` or compile from source.

To use `cargo install`:

```sh
cargo install --git https://github.com/nextonesfaster/rnd
```

`cargo` will install `rnd` in its `bin` directory, which should already be in your `PATH`.

To compile from source:

```sh
# Clone this repository
$ git clone https://github.com/nextonesfaster/rnd.git

# cd into the cloned repository
$ cd rnd

# Compile using cargo with the release flag
$ cargo build --release
```

The executable will be at `./target/release/rnd`. You can move it to your `PATH` to invoke `rnd` from any directory.

## License

`rnd` is distributed under the terms of both the MIT License and the Apache License 2.0.

See the [LICENSE-MIT][mit] and [LICENSE-APACHE][apache] files for more details.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

[rust]: https://www.rust-lang.org/tools/install
[cargo]: https://doc.rust-lang.org/cargo/getting-started/installation.html
[mit]: LICENSE-MIT
[apache]: LICENSE-APACHE
