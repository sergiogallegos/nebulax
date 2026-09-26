//! Bounded native input mailbox. Acceptance is atomic and not delivery success.
use nebulax_terminal::input::Key;
use std::collections::VecDeque;
pub const MAX_INPUT_EVENT: usize = 4096;
pub const MAX_INPUT_BYTES: usize = 16384;
pub const MAX_INPUT_EVENTS: usize = 64;
#[derive(Debug)]
pub enum Input {
    Text(String),
    Key(Key),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputError {
    Invalid,
    Full,
    Closed,
}
impl Input {
    pub fn bytes(&self) -> usize {
        match self {
            Self::Text(s) => s.len(),
            Self::Key(_) => 4,
        }
    }
    pub fn valid(&self) -> bool {
        match self {
            Self::Text(s) => {
                !s.is_empty() && s.len() <= MAX_INPUT_EVENT && !s.chars().any(char::is_control)
            }
            Self::Key(Key::Control(n)) => *n < 32,
            Self::Key(_) => true,
        }
    }
}
#[derive(Default)]
pub(crate) struct InputQueue {
    events: VecDeque<Input>,
    bytes: usize,
}
impl InputQueue {
    pub fn push(&mut self, input: Input) -> Result<(), InputError> {
        if !input.valid() {
            return Err(InputError::Invalid);
        }
        if self.events.len() == MAX_INPUT_EVENTS || self.bytes + input.bytes() > MAX_INPUT_BYTES {
            return Err(InputError::Full);
        }
        self.bytes += input.bytes();
        self.events.push_back(input);
        Ok(())
    }
    pub fn pop(&mut self) -> Option<Input> {
        let input = self.events.pop_front()?;
        self.bytes -= input.bytes();
        Some(input)
    }
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn input_caps_are_atomic_and_recover_after_consumption() {
        let mut q = InputQueue::default();
        for _ in 0..4 {
            q.push(Input::Text("x".repeat(4096))).unwrap();
        }
        assert_eq!(q.push(Input::Text("z".into())), Err(InputError::Full));
        assert_eq!(q.bytes, MAX_INPUT_BYTES);
        q.pop();
        q.push(Input::Text("z".into())).unwrap();
        for bad in ["".into(), "\x1b[5n".into(), "x".repeat(4097)] {
            assert_eq!(q.push(Input::Text(bad)), Err(InputError::Invalid));
        }
        let mut q = InputQueue::default();
        for _ in 0..64 {
            q.push(Input::Key(Key::Up)).unwrap();
        }
        assert_eq!(q.push(Input::Key(Key::Enter)), Err(InputError::Full));
        for _ in 0..64 {
            assert!(matches!(q.pop(), Some(Input::Key(Key::Up))));
        }
        assert!(q.is_empty());
        assert_eq!(q.bytes, 0);
    }
}
