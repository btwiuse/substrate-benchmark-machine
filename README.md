# The `benchmark machine` command

Different Substrate chains can have different hardware requirements.  
It is therefore important to be able to quickly gauge if a piece of hardware fits a chains' requirements.  
The `benchmark machine` command archives this by measuring key metrics and making them comparable.  

Invoking the command looks like this:  
```sh
cargo run https://github.com/btwiuse/substrate-benchmark-machine
```

## Output

The output on reference hardware:  

```pre
[INFO] 🏁 CPU score: 1.42 GiBs (✅ Blake2256: expected minimum 1.00 GiBs)                                                                                                                                        [INFO] 🏁 Memory score: 17.93 GiBs (✅ MemCopy: expected minimum 14.32 GiBs)                                                                                                                                     [INFO] 🏁 Disk score (seq. writes): 397.35 MiBs (❌ DiskSeqWrite: expected minimum 450.00 MiBs)                                                                                                                  [INFO] 🏁 Disk score (rand. writes): 193.44 MiBs (❌ DiskRndWrite: expected minimum 200.00 MiBs)                                                                                                                 [INFO] ⚠  The hardware does not meet the minimal requirements for role 'Authority'.                                                                                                                              [INFO] Running machine benchmarks...                                                                                                                                                                             [INFO]                                                                                                                                                                                                           +----------+----------------+-------------+-------------+-------------------+                                                                                                                                    | Category | Function       | Score       | Minimum     | Result            |                                                                                                                                    +===========================================================================+                                                                                                                                    | CPU      | BLAKE2-256     | 1.42 GiBs   | 1.00 GiBs   | ✅ Pass (141.3 %) |                                                                                                                                    |----------+----------------+-------------+-------------+-------------------|                                                                                                                                    | CPU      | SR25519-Verify | 726.21 KiBs | 666.00 KiBs | ✅ Pass (109.0 %) |                                                                                                                                    |----------+----------------+-------------+-------------+-------------------|
| Memory   | Copy           | 18.10 GiBs  | 14.32 GiBs  | ✅ Pass (126.4 %) |                                                                                                                                    |----------+----------------+-------------+-------------+-------------------|                                                                                                                                    | Disk     | Seq Write      | 398.56 MiBs | 450.00 MiBs | ❌ Fail ( 88.6 %) |                                                                                                                                    |----------+----------------+-------------+-------------+-------------------|
| Disk     | Rnd Write      | 193.37 MiBs | 200.00 MiBs | ✅ Pass ( 96.7 %) |
+----------+----------------+-------------+-------------+-------------------+
From 5 benchmarks in total, 4 passed and 1 failed (10% fault tolerance).
[INFO] The hardware fails to meet the requirements
Error: One of the benchmarks had a score that was lower than its requirement
```

The *score* is the average result of each benchmark. It always adheres to "higher is better".  

The *category* indicate which part of the hardware was benchmarked:  
- **CPU** Processor intensive task
- **Memory** RAM intensive task
- **Disk** Hard drive intensive task

The *function* is the concrete benchmark that was run:  
- **BLAKE2-256** The throughput of the [Blake2-256] cryptographic hashing function with 32 KiB input. The [blake2_256 function] is used in many places in Substrate. The throughput of a hash function strongly depends on the input size, therefore we settled to use a fixed input size for comparable results.
- **SR25519 Verify** Sr25519 is an optimized version of the [Curve25519] signature scheme. Signature verification is used by Substrate when verifying extrinsics and blocks.
- **Copy** The throughput of copying memory from one place in the RAM to another.
- **Seq Write** The throughput of writing data to the storage location sequentially. It is important that the same disk is used that will later-on be used to store the chain data.
- **Rnd Write** The throughput of writing data to the storage location in a random order. This is normally much slower than the sequential write.

The *score* needs to reach the *minimum* in order to pass the benchmark. This can be reduced with the `--tolerance` flag.

The *result* indicated if a specific benchmark was passed by the machine or not. The percent number is the relative score reached to the *minimum* that is needed. The `--tolerance` flag is taken into account for this decision. For example a benchmark that passes even with 95% since the *tolerance* was set to 10% would look like this: `✅ Pass ( 95.0 %)`.

## Interpretation

Ideally all results show a `Pass` and the program exits with code 0. Currently some of the benchmarks can fail even on reference hardware; they are still being improved to make them more deterministic.  
Make sure to run nothing else on the machine when benchmarking it.  
You can re-run them multiple times to get more reliable results.

## Arguments

- `--tolerance` A percent number to reduce the *minimum* requirement. This should be used to ignore outliers of the benchmarks. The default value is 10%.
- `--verify-duration` How long the verification benchmark should run.
- `--disk-duration` How long the *read* and *write* benchmarks should run each.
- `--allow-fail` Always exit the program with code 0.
- `--chain` / `--dev` Specify the chain config to use. This will be used to compare the results with the requirements of the chain (WIP).
- [`--base-path`]

License: Apache-2.0

<!-- LINKS -->
[Blake2-256]: https://www.blake2.net/
[blake2_256 function]: https://crates.parity.io/sp_core/hashing/fn.blake2_256.html
[Curve25519]: https://en.wikipedia.org/wiki/Curve25519
[`--base-path`]: ../shared/README.md#arguments
