reproduction for https://github.com/harudagondi/bevy_oddio/issues/36

1. cargo run
2. hold spacebar to record and play audio

```
thread 'Async Compute Task Pool (3)' panicked at 'Cannot build output stream.: StreamConfigNotSupported', C:\Users\danph\.cargo\registry\src\github.com-1ecc6299db9ec823\bevy_oddio-0.2.0\src\output.rs:90:10
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
thread 'Async Compute Task Pool (2)' panicked at 'Cannot build output stream.: StreamConfigNotSupported', C:\Users\danph\.cargo\registry\src\github.com-1ecc6299db9ec823\bevy_oddio-0.2.0\src\output.rs:90:10
```