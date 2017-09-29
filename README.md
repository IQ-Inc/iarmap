# `iarmapcmp`

[![Build Status](https://travis-ci.org/IQ-Inc/iarmap.svg?branch=master)](https://travis-ci.org/IQ-Inc/iarmap)

A command-line program to compare two IAR EW map files, and a library for parsing the module summary sections from the map files. This repository has no affiliation with IAR.

### Usage

```
iarmapcmp [left-map-file] [right-map-file]
```

### Features

- Parsers the "MODULE SUMMARY" table from an IAR map file
- Identifies differences between module archives
- Shows changs in object size across two map files

### Contributing

The libary and program are written in Rust. Install Rust on your system. Then, from the command line:

```
cargo install
```

Build the source with `cargo build`, run tests with `cargo test`, and generate documentation with `cargo doc [--open]`. Visual Studio Code has wonderful Rust plug-ins, including the [Rust Language Server (RLS) plugin](https://github.com/rust-lang-nursery/rls).