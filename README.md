# `indexed_state`

A Rust implementation of the `IndexedState` monad pattern from functional languages like Haskell.

## Installation

Add this git repository to your dependencies.

```toml
[dependencies.stateful]
git = "https://github.com/waynevanson/indexed-state-rust"
```

## Usage

Please see the trait `Stateful` for more details, for now.

### Summary

An `IndexedState` a structure that stores a stateful computation, where a computation is a function that takes state and returns a value and some state.

```rust
FnOnce(Input) -> (Value, Output)
```

The above signature implements `IndexedState`, so creating this in the form of a closure means the combinators on the trait are available for composition.

For example, if we were creating a PRNG, it would look like this.

```rust
use index_state::IndexedState;

fn from_usize_to_16(seed: usize) -> u16 {
    (seed % (u16::MAX as usize + 1)) as u16
}

// Implementation of an Linear Congruent Generator
fn increment(seed: usize) {
    (1164525 * seed + 1013904223) % (2**32)
}

fn prng_u16(seed: usize) {
    (from_usize_to_u16(seed), increment(seed))
}

fn main() {
    // as a function
    let prng = prng_u16;

    // as a closure
    let prng = |seed: usize| {
        (from_usize_to_u16(seed), increment(seed))
    };

    // or using the constructors and combinators
    let prng = indexed_state::new::<usize>()
        .map(from_usize_to_u16)
        .map_state(increment)


    let input_seed = 1234567890;

    // consume the state, returning the value state
    let (value, state) = prng_16.run(input_seed);

    // consume the state, returning the value only
    let value = prng_16.evaluate(input_seed)

    // consume the state, returning the state only
    let state = prng_16.execute(input_seed)
}
```

### Caveats

Closures are pure (`FnOnce`). Will consider adding a way where closures are `Fn` so that `run` can be called multiple times on a structure.
