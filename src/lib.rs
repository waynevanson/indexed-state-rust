use core::marker::PhantomData;

/// Allows the construction of computations that hold state; the indexed state monad pattern.
/// Given an input state of `I`, it can return the value of `A` whilst keeping any changes to the output state as `O`.
pub trait Stateful<I, O, A>: Sized {
    /// Given an input state, returns an output state as `O` and the inner value as `A`.
    /// All composable functions are derived from this function.
    ///
    /// To use only `A` or `O`, consider using `evaluate` or `execute` function respectively.
    fn run(self, state: I) -> (A, O);

    /// Consumes the input state and returns only the inner value as `A`.
    fn evaluate(self, state: I) -> A {
        self.run(state).0
    }

    /// Consumes the input state and returns only the output state as `O`.
    fn execute(self, state: I) -> O {
        self.run(state).1
    }

    /// Applies a covariant function to `A` that goes from `A` to `B`.
    fn map<F, B>(self, closure: F) -> Map<Self, F, A>
    where
        F: FnOnce(A) -> B,
    {
        Map {
            stateful: self,
            closure,
            phantom: PhantomData,
        }
    }

    /// Applies a function that goes from `A` to a new `Stateful` structure.
    /// This allows composing two `Stateful` structures, where the value of the first
    /// is used as a parameter for the second.
    ///
    /// This is equivilent to a monadic bind in functional languages.
    fn and_then<F, U, P, B>(self, kleisli: F) -> AndThen<Self, F, (A, O)>
    where
        U: Stateful<O, P, B>,
        F: FnOnce(A) -> U,
    {
        AndThen {
            stateful: self,
            kleisli,
            phantom: PhantomData,
        }
    }

    /// Applies a covariant function to the output state, that goes from `O` to `P`.
    fn map_state<F, P>(self, closure: F) -> MapState<Self, F, O>
    where
        F: FnOnce(O) -> P,
    {
        MapState {
            stateful: self,
            closure,
            phantom: PhantomData,
        }
    }

    /// Applies a contravariant function to the input state that goes from `K` to `I`.
    /// This changes the input state to be `K` instead of `I`,
    fn contramap_state<F, K>(self, contravariant: F) -> ContramapState<Self, F>
    where
        F: FnOnce(K) -> I,
    {
        ContramapState {
            stateful: self,
            contravariant,
        }
    }
}

impl<I, O, A, T> Stateful<I, O, A> for T
where
    T: FnOnce(I) -> (A, O),
{
    fn run(self, state: I) -> (A, O) {
        self(state)
    }
}

/// Create a `Stateful` structure given the input type can be cloned.
pub fn new<I>() -> impl Stateful<I, I, I>
where
    I: Clone,
{
    |state: I| (state.clone(), state)
}

pub fn gets<I, O>(covariant: impl FnOnce(I) -> O) -> impl Stateful<I, O, O>
where
    O: Clone,
{
    |state: I| {
        let output = covariant(state);
        (output.clone(), output)
    }
}

pub fn gots<I, A>(covariant: impl FnOnce(I) -> A) -> impl Stateful<I, I, A>
where
    I: Clone,
{
    |state: I| {
        let value = covariant(state.clone());
        (value, state)
    }
}

pub struct Map<T, F, A> {
    stateful: T,
    closure: F,
    phantom: PhantomData<A>,
}

impl<I, O, B, T, F, A> Stateful<I, O, B> for Map<T, F, A>
where
    T: Stateful<I, O, A>,
    F: FnOnce(A) -> B,
{
    fn run(self, state: I) -> (B, O) {
        let (a, o) = self.stateful.run(state);
        let b = (self.closure)(a);
        (b, o)
    }
}

pub struct MapState<T, F, O> {
    stateful: T,
    closure: F,
    phantom: PhantomData<O>,
}

impl<I, O, P, T, F, A> Stateful<I, P, A> for MapState<T, F, O>
where
    T: Stateful<I, O, A>,
    F: FnOnce(O) -> P,
{
    fn run(self, state: I) -> (A, P) {
        let (a, o) = self.stateful.run(state);
        let p = (self.closure)(o);
        (a, p)
    }
}

pub struct AndThen<T, F, A> {
    stateful: T,
    kleisli: F,
    phantom: PhantomData<A>,
}

impl<I, P, A, T, F, O, U, B> Stateful<I, P, B> for AndThen<T, F, (A, O)>
where
    T: Stateful<I, O, A>,
    U: Stateful<O, P, B>,
    F: FnOnce(A) -> U,
{
    fn run(self, state: I) -> (B, P) {
        let (a, o) = self.stateful.run(state);
        (self.kleisli)(a).run(o)
    }
}

pub struct ContramapState<T, F> {
    stateful: T,
    contravariant: F,
}

impl<I, O, A, K, T, F> Stateful<K, O, A> for ContramapState<T, F>
where
    T: Stateful<I, O, A>,
    F: FnOnce(K) -> I,
{
    fn run(self, state: K) -> (A, O) {
        let state = (self.contravariant)(state);
        self.stateful.run(state)
    }
}
