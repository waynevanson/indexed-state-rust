use core::marker::PhantomData;

/// Allows the construction of computations that hold state; the indexed state monad pattern.
/// Given an input state of `I`, it can return the value of `A` whilst keeping any changes to the output state as `O`.
pub trait Stateful<Input, Output, Value>: Sized {
    /// Given an input state, returns an output state as `O` and the inner value as `A`.
    /// All composable functions are derived from this function.
    ///
    /// To use only `A` or `O`, consider using `evaluate` or `execute` function respectively.
    fn run(self, state: Input) -> (Value, Output);

    /// Consumes the input state and returns only the inner value as `A`.
    fn evaluate(self, state: Input) -> Value {
        self.run(state).0
    }

    /// Consumes the input state and returns only the output state as `O`.
    fn execute(self, state: Input) -> Output {
        self.run(state).1
    }

    /// Applies a covariant function to `A` that goes from `A` to `B`.
    fn map<Function, ValueNext>(self, closure: Function) -> Map<Self, Function, Value>
    where
        Function: FnOnce(Value) -> ValueNext,
    {
        Map {
            first: self,
            covariant: closure,
            phantom: PhantomData,
        }
    }

    /// Applies a function that goes from `A` to a new `Stateful` structure.
    /// This allows composing two `Stateful` structures, where the value of the first
    /// is used as a parameter for the second.
    ///
    /// This is equivilent to a monadic bind in functional languages.
    fn and_then<Covariant, Second, SecondOutput, SecondValue>(
        self,
        kleisli: Covariant,
    ) -> AndThen<Self, Covariant, (Value, Output)>
    where
        Second: Stateful<Output, SecondOutput, SecondValue>,
        Covariant: FnOnce(Value) -> Second,
    {
        AndThen {
            stateful: self,
            kleisli,
            phantom: PhantomData,
        }
    }

    /// Applies a covariant function to the output state, that goes from `O` to `P`.
    fn map_state<Covariant, SecondState>(
        self,
        closure: Covariant,
    ) -> MapState<Self, Covariant, Output>
    where
        Covariant: FnOnce(Output) -> SecondState,
    {
        MapState {
            first: self,
            covariant: closure,
            phantom: PhantomData,
        }
    }

    /// Applies a contravariant function to the input state that goes from `K` to `I`.
    /// This changes the input state to be `K` instead of `I`,
    fn contramap_state<Covariant, FirstInput>(
        self,
        contravariant: Covariant,
    ) -> ContramapState<Self, Covariant>
    where
        Covariant: FnOnce(FirstInput) -> Input,
    {
        ContramapState {
            first: self,
            contravariant,
        }
    }
}

impl<FirstInput, SecondInput, FirstValue, Covariant> Stateful<FirstInput, SecondInput, FirstValue>
    for Covariant
where
    Covariant: FnOnce(FirstInput) -> (FirstValue, SecondInput),
{
    fn run(self, state: FirstInput) -> (FirstValue, SecondInput) {
        self(state)
    }
}

/// Create a `Stateful` structure given the input type can be cloned.
pub fn new<State>() -> impl Stateful<State, State, State>
where
    State: Clone,
{
    |state: State| (state.clone(), state)
}

pub fn gets<Input, Output>(
    covariant: impl FnOnce(Input) -> Output,
) -> impl Stateful<Input, Output, Output>
where
    Output: Clone,
{
    |state: Input| {
        let output = covariant(state);
        (output.clone(), output)
    }
}

pub fn gots<Input, Value>(
    covariant: impl FnOnce(Input) -> Value,
) -> impl Stateful<Input, Input, Value>
where
    Input: Clone,
{
    |state: Input| {
        let value = covariant(state.clone());
        (value, state)
    }
}

pub struct Map<First, Covariant, Phantom> {
    first: First,
    covariant: Covariant,
    phantom: PhantomData<Phantom>,
}

impl<FirstInput, SecondInput, SecondValue, First, Covariant, FirstValue>
    Stateful<FirstInput, SecondInput, SecondValue> for Map<First, Covariant, FirstValue>
where
    First: Stateful<FirstInput, SecondInput, FirstValue>,
    Covariant: FnOnce(FirstValue) -> SecondValue,
{
    fn run(self, state: FirstInput) -> (SecondValue, SecondInput) {
        let (a, o) = self.first.run(state);
        let b = (self.covariant)(a);
        (b, o)
    }
}

pub struct MapState<First, Covariant, Phantom> {
    first: First,
    covariant: Covariant,
    phantom: PhantomData<Phantom>,
}

impl<FirstInput, SecondInput, SecondOutput, First, Function, FirstValue>
    Stateful<FirstInput, SecondOutput, FirstValue> for MapState<First, Function, SecondInput>
where
    First: Stateful<FirstInput, SecondInput, FirstValue>,
    Function: FnOnce(SecondInput) -> SecondOutput,
{
    fn run(self, state: FirstInput) -> (FirstValue, SecondOutput) {
        let (a, o) = self.first.run(state);
        let p = (self.covariant)(o);
        (a, p)
    }
}

pub struct AndThen<First, Kleisli, Phantom> {
    stateful: First,
    kleisli: Kleisli,
    phantom: PhantomData<Phantom>,
}

impl<FirstInput, SecondOutput, FirstValue, First, Kleisli, SecondInput, Second, SecondValue>
    Stateful<FirstInput, SecondOutput, SecondValue>
    for AndThen<First, Kleisli, (FirstValue, SecondInput)>
where
    First: Stateful<FirstInput, SecondInput, FirstValue>,
    Second: Stateful<SecondInput, SecondOutput, SecondValue>,
    Kleisli: FnOnce(FirstValue) -> Second,
{
    fn run(self, state: FirstInput) -> (SecondValue, SecondOutput) {
        let (a, o) = self.stateful.run(state);
        (self.kleisli)(a).run(o)
    }
}

pub struct ContramapState<First, Contravariant> {
    first: First,
    contravariant: Contravariant,
}

impl<Input, Output, Value, PreviousInput, Second, Contravariant>
    Stateful<PreviousInput, Output, Value> for ContramapState<Second, Contravariant>
where
    Second: Stateful<Input, Output, Value>,
    Contravariant: FnOnce(PreviousInput) -> Input,
{
    fn run(self, state: PreviousInput) -> (Value, Output) {
        let state = (self.contravariant)(state);
        self.first.run(state)
    }
}

pub struct Apply<First, Second, Phantom> {
    first: First,
    second: Second,
    phantom: PhantomData<Phantom>,
}

impl<FirstInput, SecondInput, SecondOutput, FirstValue, Covariant, First, Second, SecondValue>
    Stateful<FirstInput, SecondOutput, SecondValue>
    for Apply<First, Second, (SecondInput, FirstValue, Covariant)>
where
    First: Stateful<FirstInput, SecondInput, Covariant>,
    Second: Stateful<SecondInput, SecondOutput, FirstValue>,
    Covariant: FnOnce(FirstValue) -> SecondValue,
{
    fn run(self, state: FirstInput) -> (SecondValue, SecondOutput) {
        let (f, state) = self.first.run(state);
        let (a, p) = self.second.run(state);
        let b = f(a);
        (b, p)
    }
}
