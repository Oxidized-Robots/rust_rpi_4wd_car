# rust_rpi_4wd_car
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-v2.0%20adopted-ff69b4.svg)](CODE_OF_CONDUCT.md)
[![Crates.io](https://img.shields.io/crates/l/rust_rpi_4wd_car)](https://github.com/Oxidized-Robots/rust_rpi_4wd_car/blob/main/README.md#licenses)
<br>
Rust code for Yahboom 4WD smart robot for Raspberry Pi 4B.

<img src="https://avatars.githubusercontent.com/u/82801111?s=200&v=4" width=64 alt="Robotic avatar icon of Oxidized Robots">

This is the founding project of [Oxidized Robots] on Github.

## Table Of Contents

* [Getting Started](#getting-started)
* [Using The Crate](#using-the-crate)
* [Examples](#examples)
* [Contributing](#contributing)
* [Licenses](#licenses)

## Getting Started

You will need to have a recent version of [Rust] installed.
Any version of Rust that supports version 0.12.0 or later of [rppal] should
work but the release version 1.51.0 of Rust have been used during initial
development.
Earlier versions might work as well but have not been tested.

Development can be done on any OS that Rust supports but the only expected
output target is a Raspberry Pi running a Linux OS.
All initial development has been done with a combination of a laptop running
Windows 10 and a 4GB Raspberry Pi 4 running the Raspberry Pi OS (Raspbian).

## Using The Crate

To use the crate in your own project all you need to do is include it in
`[dependencies]` of you project like you would any other crate.
If you have [cargo-edit] install then on the command line you can use:

```shell script
cargo add rust_rpi_4wd_car
```

Which should add something like this in your [Cargo.toml]:

```toml
[dependencies]
rust_rpi_4wd_car = "0.0.10"
```

## Examples

You will find several examples in the `examples` directory. The `demo` one
is a good place to start with as it uses most aspects of the crate.

To build `demo` start by clone this project somewhere on your Raspberry Pi:

```shell
git clone https://github.com/Oxidized-Robots/rust_rpi_4wd_car
```

Change directory into the new one just created:

```shell
cd rust_rpi_4wd_car
```

Next execute the follow to build and run the `demo`:

```shell
cargo run --example demo
```

You should see the series of tests being run which demo most of the aspects from
the crate.

You can find the latest release version by go to [rust_rpi_4wd_car] on the
[crates.io] website or for development versions, bug reports, etc please go to
the [Github repository] of the project.

## Contributing

Contributors are welcome.
Please note that this project has a [Contributor Covenant Code of Conduct].
By participating in this project you agree to abide by its terms.

All intentionally contributed code will be considered to also be contributed
under a dual licensing of [APACHE] and [MIT] without any additional terms or
conditions.
Please include your information in a comment on all code files for the copyright
etc.

All intentionally contributed documentation or non-code text like this README
etc. will be considered to be contributed under the same [CC-BY-SA] license
without any additional terms or conditions.

Pull requests are always welcome. For major changes, please open an issue first
to discuss what you would like to change.
Please make sure to update or add tests as appropriate.

## Licenses

All code is licensed under both of the following:

  * [APACHE] - Apache License, Version 2.0
  * [MIT] - MIT License

and may be used with either or both at your option.

You can find copies of the licenses in the [LICENSE-APACHE] and the
[LICENSE-MIT] files.
All documentation like this README is licensed under the Creative Commons
Attribution-ShareAlike 4.0 International License (CC-BY-SA).
You can find a copy of the [CC-BY-SA] license in the [LICENSE-CC-BY-SA] file.

[APACHE]: https://opensource.org/licenses/Apache-2.0
[CC-BY-SA]: http://creativecommons.org/licenses/by-sa/4.0/
[Cargo.toml]: https://doc.rust-lang.org/cargo/guide/dependencies.html
[Contributor Covenant Code of Conduct]: CODE_OF_CONDUCT.md
[Github repository]: https://github.com/Oxidized-Robots/rust_rpi_4wd_car
[LICENSE-APACHE]: LICENSE-APACHE
[LICENSE-CC-BY-SA]: LICENSE-CC-BY-SA
[LICENSE-MIT]: LICENSE-MIT
[MIT]: https://opensource.org/licenses/MIT
[Oxidized Robots]: https://github.com/Oxidized-Robots
[Rust]: https://www.rust-lang.org/
[cargo-edit]: https://crates.io/crates/cargo-edit
[crates.io]: https://crates.io
[rppal]: https://crates.io/crates/rppal
[rust_rpi_4wd_car]: https://crates.io/crates/rust_rpi_4wd_car

<hr>
Copyright &copy; 2021, Michael Cummings<br/>
<a rel="license" href="https://creativecommons.org/licenses/by-sa/4.0/">
<img alt="Creative Commons License" style="border-width:0" src="https://i.creativecommons.org/l/by-sa/4.0/88x31.png" />
</a>
<div>Icons made by <a href="https://www.freepik.com" title="Freepik">Freepik</a> from <a href="https://www.flaticon.com/" title="Flaticon">www.flaticon.com</a></div>
