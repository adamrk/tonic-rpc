# Tonic-RPC

A macro that allows you to use [`tonic`](https://crates.io/crates/tonic) by defining your RPC interfaces in pure Rust code instead of using [`protobuf`](https://developers.google.com/protocol-buffers). When your service doesn't have clients written in other languages this allows you use the full power of the Rust type system in you RPC interface.

# Examples
Several examples can be found in the [`tests`](https://github.com/adamrk/tonic-rpc/tree/main/tonic-rpc/tests) folder.
