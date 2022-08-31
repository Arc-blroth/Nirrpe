use std::fmt::{Debug, Formatter};

use lasso::{Rodeo, Spur};
use pest::iterators::Pair;
use unicode_normalization::{is_nfc_quick, IsNormalized, UnicodeNormalization};

use crate::parse::{Parseable, Rule};

/// An interned, NFC-normalized Unicode string.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Ident {
    id: Spur,
}

impl Ident {
    pub fn new<S: AsRef<str>>(rodeo: &mut Rodeo, name: S) -> Self {
        let name = name.as_ref();
        if is_nfc_quick(name.chars()) == IsNormalized::Yes {
            Self {
                id: rodeo.get_or_intern(name),
            }
        } else {
            Self {
                id: rodeo.get_or_intern(name.nfc().collect::<String>()),
            }
        }
    }

    pub fn resolve<'a>(&self, rodeo: &'a Rodeo) -> &'a str {
        rodeo.resolve(&self.id)
    }
}

impl Parseable for Ident {
    fn parse(rodeo: &mut Rodeo, pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::IDENT);
        Self::new(rodeo, pair.as_str())
    }
}

impl Debug for Ident {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ident").field("id", &self.id.into_inner()).finish()
    }
}
