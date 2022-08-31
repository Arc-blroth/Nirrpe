use pest::iterators::Pair;
use pest::RuleType;

pub(crate) trait GetSingleInner<'i, R> {
    fn into_single_inner(self) -> Pair<'i, R>;
}

impl<'i, R: RuleType> GetSingleInner<'i, R> for Pair<'i, R> {
    fn into_single_inner(self) -> Pair<'i, R> {
        self.into_inner().next().expect("Expected single inner rule!")
    }
}
