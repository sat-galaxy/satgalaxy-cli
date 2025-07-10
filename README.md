# satgalaxy-cli 🚀

**A powerful, cross-platform command-line interface for SAT solving, powered by `satgalaxy-rs`.**

 ---

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/badge/Build-Passing-brightgreen)](https://github.com/your-username/satgalaxy-rs/actions) 
---
### Platform & Build Compatibility

![Linux](https://img.shields.io/badge/OS-Linux-informational?logo=linux&logoColor=white)
![macOS](https://img.shields.io/badge/OS-macOS-informational?logo=apple&logoColor=white)
![Windows](https://img.shields.io/badge/OS-Windows-informational?logo=windows&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-informational?logo=rust&logoColor=white)
---

## 🌟 Overview

satgalaxy-cli brings the robust SAT solving capabilities of satgalaxy-rs (and its underlying satgalaxy-core C library with Minisat and Glucose) to your command line. This tool allows you to:

- **Solve SAT problems directly**: Feed DIMACS CNF files and get solutions.
- **Full Command-Line Argument Support**: Access nearly all original command-line options for `Minisat` and `Glucose` solvers, giving you fine-grained control over their behavior.
- **Network File Reading**: Solve problems from local files or directly from remote URLs (e.g., `http://`, `https://`, `ftp://`).
- `Cross-Platform`: Built with Rust, `satgalaxy-cli` provides a single executable that works across Linux, macOS, and Windows.

Whether you're a researcher, a student, or just need to quickly solve a SAT instance, `satgalaxy-cli` offers a convenient and powerful solution.

## 🚀 Getting Started

### Prerequisites

To build and run `satgalaxy-cli`, you'll need:

- **Rust Toolchain**: Install Rust via rustup (https://rustup.rs/).
- **CMake**: The underlying `satgalaxy-core` dependency uses CMake for its build system, so ensure it's installed on your system.
- **C/C++ Compiler**: A C/C++ compiler (like GCC, Clang, MSVC) compatible with your system is required to compile `satgalaxy-core`.

## Installation (from source)

Since satgalaxy-cli is not yet on crates.io, you can build and install it directly from its source code:

1. Clone the repository:
```bash
git clone https://github.com/sat-galaxy/satgalaxy-cli.git
cd satgalaxy-cli
```

2. Build and install:
```bash
cargo install --path .
```

This command compiles the project and places the `satgalaxy` executable in your Cargo bin directory (usually `~/.cargo/bin`), making it available in your system's PATH.

Note: If you're using Windows, you might need to add the `~/.cargo/bin` directory to your PATH environment variable.

## 📖 Usage

`satgalaxy-cli` provides a simple and intuitive command-line interface.

```
A command line interface for the multi sat solver

Usage: satgalaxy <COMMAND>

Commands:
  minisat  Use minisat(2.2.0) solver https://github.com/niklasso/minisat
  glucose  Use glucose(4.2.1) solver https://github.com/audemard/glucose
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Examples

Here's how to use `satgalaxy-cli` in practice:

#### Solving a Local DIMACS CNF File

To solve a SAT problem from a local file, specify the solver command followed by the file path.
```bash
# Using Minisat
satgalaxy minisat my_problem.cnf

# Using Glucose
satgalaxy glucose another_problem.cnf
```
#### Solving a Problem from a URL

`satgalaxy-cli` can directly fetch and solve problems from a URL (e.g., `http://`, `https://`, `ftp://`).

```bash
# Using Minisat to solve a problem from a URL
satgalaxy minisat https://benchmark-database.de/file/000a41cdca43be89ed62ea3abf2d0b64?context=cnf

# Using Glucose to solve a compressed problem
satgalaxy glucose https://benchmark-database.de/file/000a41cdca43be89ed62ea3abf2d0b64?context=cnf
```

#### Passing Solver-Specific Options

You can pass arguments directly to the underlying Minisat or Glucose solver by adding them
```bash
satgalaxy glucose my_problem.cnf  --K=0.5

satgalaxy minisat another_problem.cnf  --var-decay=0.5
```

To see the full set of command-line options available for a specific solver, use the `--help` flag with that solver's subcommand:
```bash
satgalaxy minisat --help
satgalaxy glucose --help
```

## 📜 License

This project is distributed under the MIT License.

## 📧 Contact

If you have any questions, suggestions, or just want to chat about SAT solvers, feel free to open an Issue or reach out. We'd love to hear from you!