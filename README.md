## tsql

**tsql** is a syntax for querying data off of a tree-sitter
[`Node`](https://docs.rs/tree-sitter/latest/tree_sitter/struct.Node.html)
instance.

We provide a single trait
[`NodeExt`](https://github.com/terror/tsql/blob/f7738f8002c3f15c165224c39e72b3ae3e88e044/src/lib.rs#L14)
which implements a single method
[`find`](https://github.com/terror/tsql/blob/f7738f8002c3f15c165224c39e72b3ae3e88e044/src/lib.rs#L19),
capable of parsing and executing a rich query syntax for grabbing information
off of a
[`Node`](https://docs.rs/tree-sitter/latest/tree_sitter/struct.Node.html).

## Installation

You can add `tsql` to your project using [cargo](https://doc.rust-lang.org/cargo/index.html),
the Rust package manager:

```
cargo add tsql
```
