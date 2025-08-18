## tsql

**tsql** is a syntax for querying data off of a treesiter [`Node`](https://docs.rs/tree-sitter/latest/tree_sitter/struct.Node.html) instance.

We provide a single trait, i.e. `NodeExt`, which provides a single method
`find`, capable of parsing and executing a rich query syntax for grabbing
information off of a `Node`.
