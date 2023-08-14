# jaro_winkler

A fast implementation of [Jaro-Winkler distance] comparing two
`&str` values.

[Jaro-Winkler distance]: https://en.wikipedia.org/wiki/Jaro%E2%80%93Winkler_distance

## Usage

Add `jaro_winkler` to your `Cargo.toml`:

```toml
[dependencies]
jaro_winkler = "0.1.0"
```

## Benchmarks

See [benches/bench.rs](benches/bench.rs).

Comparing different lengths results in different execution times.

On my 2021 M1 Mac, benchmark results compared against [strsim] and [eddie]:

| character lengths | jaro_winkler | strsim  | eddie   |
|-------------------|--------------|---------|---------|
| 9, 10             | 40ns         | 90ns    | 102ns   |
| 4, 5              | 19ns         | 47ns    | 82ns    |
| 4, 25             | 21ns         | 106ns   | 97ns    |
| 223, 29           | 498ns        | 2815ns  | 1168ns  |
| 223, 188          | 10147ns      | 25195ns | 12080ns |

[strsim]: https://github.com/dguo/strsim-rs
[eddie]: https://github.com/thaumant/eddie

## Copyright

Copyright 2022 Josh Clayton. See the [LICENSE](LICENSE).
