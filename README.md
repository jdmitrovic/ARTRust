# rustART

rustART is library written in Rust which implements [Adaptive Radix Trees](https://db.in.tum.de/~leis/papers/ART.pdf). Currently,
only CRUD operations are implemented.

## Testing and benchmarking

Unit tests can be run with `cargo test` command, and benchmarks can be run with `cargo bench`
command.

There are two benchmarks: first compares elapsed times for insertion, and the second compares elapsed time
for deletion of mapped values.

