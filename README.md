Rust Blocks
===========

A lightweight implementation for an i3 status bar written in Rust.

See the [i3bar protocol](https://i3wm.org/docs/i3bar-protocol.html) for details
on the protocol.

Installation
------------

Implementation
--------------

Each block is represented by an infinite loop sending a `(name, text)` struct
through a channel.
