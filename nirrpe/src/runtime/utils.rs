use std::fmt::{Debug, Display, Formatter};

pub struct DelegateDebugToDisplay<'a, T>(pub &'a T);

impl<'a, T: Display> Debug for DelegateDebugToDisplay<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.0, f)
    }
}
