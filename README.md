# ds - A modern directory lister

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

`ds` (Directory List) is a fast, modern replacement for the `ls` command, written in Rust. It provides colorful output, additional file information, and customizable display options while maintaining compatibility with traditional `ls` usage patterns.

![Demo Screenshot](./photo/2025-07-27-193214_hyprshot.jpg)

## Features

- 🎨 **Colorful output** with syntax highlighting for different file types
- 📊 **Multiple display formats**: permissions, sizes, timestamps, owners
- 🔍 **Advanced filtering**: show hidden files, directories only, respect .gitignore
- 🔄 **Flexible sorting**: by name, size, extension, timestamps, etc.
- ⚡ **Blazing fast** - written in Rust for maximum performance
- 📁 **Simple installation** - single binary with no dependencies

## Installation

### From source (requires Rust toolchain)
```bash
$ cargo install --git https://github.com/Mr-Fox-h/ds 
```
