use std::fmt::Debug;

pub trait Reducer<S, A>: Send + Sync {
    fn reduce(&self, state: &mut S, action: A);
}

pub struct ReducerTester<S, A> {
    state: S,
    actions: Vec<A>,
    errors: Vec<ReducerError>,
}

#[derive(Debug, Clone)]
pub struct ReducerError {
    pub action: String,
    pub error: String,
}

impl<S: Debug + Clone, A: Debug + Clone> ReducerTester<S, A> {
    pub fn new(initial_state: S) -> Self {
        Self {
            state: initial_state,
            actions: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn dispatch<R: Reducer<S, A>>(&mut self, reducer: &R, action: A) -> &mut Self {
        let action_desc = format!("{:?}", action);
        let state_before = format!("{:?}", self.state);

        reducer.reduce(&mut self.state, action.clone());
        self.actions.push(action);

        let state_after = format!("{:?}", self.state);
        if state_before == state_after && !self.actions.is_empty() {
            self.errors.push(ReducerError {
                action: action_desc,
                error: format!("State did not change after action. State: {}", state_before),
            });
        }

        self
    }

    pub fn dispatch_many<R: Reducer<S, A>>(
        &mut self,
        reducer: &R,
        actions: impl IntoIterator<Item = A>,
    ) -> &mut Self {
        for action in actions {
            self.dispatch(reducer, action);
        }
        self
    }

    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn actions(&self) -> &[A] {
        &self.actions
    }

    pub fn errors(&self) -> &[ReducerError] {
        &self.errors
    }

    pub fn assert_state<F>(&self, predicate: F) -> Result<(), String>
    where
        F: FnOnce(&S) -> bool,
    {
        if predicate(&self.state) {
            Ok(())
        } else {
            Err(format!(
                "State assertion failed. Final state: {:?}",
                self.state
            ))
        }
    }

    pub fn assert_no_errors(&self) -> Result<(), String> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            let error_msgs: Vec<String> = self
                .errors
                .iter()
                .map(|e| format!("  - {}: {}", e.action, e.error))
                .collect();
            Err(format!(
                "Reducer produced {} error(s):\n{}",
                self.errors.len(),
                error_msgs.join("\n")
            ))
        }
    }

    pub fn verify_invariant<F>(&self, invariant: F) -> Result<(), String>
    where
        F: Fn(&S) -> bool,
    {
        if invariant(&self.state) {
            Ok(())
        } else {
            Err(format!("Invariant violated. Final state: {:?}", self.state))
        }
    }
}

pub struct TableDrivenCase<S, A> {
    pub name: String,
    pub initial_state: S,
    pub action: A,
    pub expected_state: S,
    pub assertion: Option<Box<dyn Fn(&S, &S) -> bool>>,
}

pub fn test_table_driven<S, A, R>(
    reducer: &R,
    cases: Vec<TableDrivenCase<S, A>>,
) -> Vec<(String, Result<(), String>)>
where
    S: Clone + Debug + PartialEq,
    A: Clone + Debug,
    R: Reducer<S, A>,
{
    cases
        .into_iter()
        .map(|case| {
            let result = (|| {
                let mut tester = ReducerTester::new(case.initial_state);
                tester.dispatch(reducer, case.action);

                if let Some(assertion) = case.assertion {
                    if !assertion(tester.state(), &case.expected_state) {
                        return Err(format!(
                            "Assertion failed for '{}'. Got: {:?}, Expected: {:?}",
                            case.name,
                            tester.state(),
                            case.expected_state
                        ));
                    }
                } else if tester.state() != &case.expected_state {
                    return Err(format!(
                        "State mismatch for '{}'. Got: {:?}, Expected: {:?}",
                        case.name,
                        tester.state(),
                        case.expected_state
                    ));
                }

                tester.assert_no_errors()?;
                Ok(())
            })();

            (case.name, result)
        })
        .collect()
}

#[allow(unused_imports)]
pub mod prelude {
    pub use super::{test_table_driven, Reducer, ReducerError, ReducerTester, TableDrivenCase};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct CounterState {
        count: i32,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum CounterAction {
        Increment,
        Decrement,
        Reset,
        Add(i32),
    }

    struct CounterReducer;

    impl Reducer<CounterState, CounterAction> for CounterReducer {
        fn reduce(&self, state: &mut CounterState, action: CounterAction) {
            match action {
                CounterAction::Increment => state.count += 1,
                CounterAction::Decrement => state.count -= 1,
                CounterAction::Reset => state.count = 0,
                CounterAction::Add(n) => state.count += n,
            }
        }
    }

    #[test]
    fn test_reducer_tester_basic() {
        let mut tester = ReducerTester::new(CounterState { count: 0 });
        let reducer = CounterReducer;

        tester.dispatch(&reducer, CounterAction::Increment);
        tester.dispatch(&reducer, CounterAction::Increment);
        tester.dispatch(&reducer, CounterAction::Decrement);

        assert_eq!(tester.state().count, 1);
        assert_eq!(tester.actions().len(), 3);
    }

    #[test]
    fn test_reducer_tester_errors_on_no_change() {
        let mut tester = ReducerTester::new(CounterState { count: 5 });
        let reducer = CounterReducer;

        tester.dispatch(&reducer, CounterAction::Reset);
        tester.dispatch(&reducer, CounterAction::Reset);

        assert!(!tester.errors().is_empty());
    }

    #[test]
    fn test_table_driven_cases() {
        let reducer = CounterReducer;
        let cases = vec![
            TableDrivenCase {
                name: "increment from zero".to_string(),
                initial_state: CounterState { count: 0 },
                action: CounterAction::Increment,
                expected_state: CounterState { count: 1 },
                assertion: None,
            },
            TableDrivenCase {
                name: "decrement from one".to_string(),
                initial_state: CounterState { count: 1 },
                action: CounterAction::Decrement,
                expected_state: CounterState { count: 0 },
                assertion: None,
            },
            TableDrivenCase {
                name: "reset to zero".to_string(),
                initial_state: CounterState { count: 100 },
                action: CounterAction::Reset,
                expected_state: CounterState { count: 0 },
                assertion: None,
            },
            TableDrivenCase {
                name: "add positive".to_string(),
                initial_state: CounterState { count: 0 },
                action: CounterAction::Add(5),
                expected_state: CounterState { count: 5 },
                assertion: None,
            },
        ];

        let results = test_table_driven(&reducer, cases);

        for (name, result) in &results {
            assert!(result.is_ok(), "Case '{}' failed: {:?}", name, result);
        }
    }

    #[test]
    fn test_custom_assertion() {
        let reducer = CounterReducer;
        let cases = vec![TableDrivenCase {
            name: "always positive".to_string(),
            initial_state: CounterState { count: 0 },
            action: CounterAction::Decrement,
            expected_state: CounterState { count: -1 },
            assertion: Some(Box::new(|got, _| got.count >= 0)),
        }];

        let results = test_table_driven(&reducer, cases);
        let (_, result) = &results[0];
        assert!(result.is_err());
    }
}
