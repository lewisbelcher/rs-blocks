Rust Blocks
===========

[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![pipeline](https://gitlab.com/lewisbelcher/rs-blocks/badges/master/pipeline.svg)](https://gitlab.com/lewisbelcher/rs-blocks/pipelines)
[![crate](https://img.shields.io/crates/v/rs-blocks.svg)](https://crates.io/crates/rs-blocks)

A lightweight implementation for an i3 status bar written in Rust.

See the [i3bar protocol](https://i3wm.org/docs/i3bar-protocol.html) for details
on the protocol.

Installation
------------

1. [Get Rust](https://www.rust-lang.org/tools/install)
2. Clone this repo (optional)
3. Run `cargo install --path <repo path>` (if you did step 3) or `cargo install rs-blocks`
4. Use `rs-blocks`!

Configuration
-------------

`rs-blocks` will look for `$XDG_CONFIG_HOME/rs-blocks/config` for a config file.
Alternatively, a config file can be specified by using the argument `--config`.

Config files should be in [toml format](https://github.com/toml-lang/toml). The
section title must match an available block type, and the remaining config within
each section will be used to configure that block.

Multiple entries of the same block can be given as long as they specify different
`name`s. E.g.:

    [time]
    name = "time1"
    format = "%D"

    [time]
    name = "time2"
    format = "%S"

Blocks will appear in the order they are given in the config file.

Available blocks are:
  * Battery
  * Brightness (using `brightnessctl`)
  * CPU
  * Memory
  * Network
  * Time
  * Volume (using `amixer`)

Implementation
--------------

Blocks are represented by infinite loops in threads sending a `(name, text)` structs
through a channel, received on the main thread. The trait `Configure` should be
implemented for setting up a block (a toml string is passed as the config argument)
and the `Sender` trait should be implemented for creating a message sending
function (see these traits for details).
