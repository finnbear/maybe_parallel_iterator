# maybe_parallel_iterator

Write your code once. Then toggle between sequential and parallel iterators with a feature flag!

```rust
let a: Vec<i32> = (0..100).collect();
a.into_maybe_parallel_iterator()
    .with_min_sequential(2)
    .map(|n| -n)
    .enumerate()
    .flat_map(|(e, n)| vec![e as i32, n, n + 1000].into_maybe_parallel_iterator())
    .for_each(|item| {
        println!("par: {:?}", item);
    })
```

## Features

- `into_maybe_par_iter`
- `maybe_par_iter`
- `maybe_par_iter_mut`

Use the `rayon` feature flag to enable [rayon](https://github.com/rayon-rs/rayon) parallelism.

The default is sequential ("rayoff" mode).

## Limitations

For now, only supports:

- *`enumerate`
- `for_each`
- `map`
- `flat_map`
- *`with_min_sequential` (no-op unless `rayon` feature enabled)

*Only available for `IndexedParallelIterator`'s if `rayon` feature enabled.

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.