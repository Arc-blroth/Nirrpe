use std::fmt::{Debug, Formatter};

use unicode_normalization::{is_nfc_quick, IsNormalized, UnicodeNormalization};

/// A NFC-normalized Unicode string.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Ident {
    pub id: String,
}

impl Ident {
    pub fn new<S: AsRef<str>>(name: S) -> Self {
        let name = name.as_ref();
        if is_nfc_quick(name.chars()) == IsNormalized::Yes {
            Self { id: name.to_string() }
        } else {
            Self {
                id: name.nfc().collect::<String>(),
            }
        }
    }
}

impl Debug for Ident {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Ident").field(&self.id).finish()
    }
}
