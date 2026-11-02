//! Scoped immutable rule fixtures for engine scenarios, absent from production.
//!
//! Only executable rules are overlaid. Physical names, types, colors, printed
//! costs and admission remain the selected fixture card's real metadata. Keep
//! the guard alive for every game, branch and restored snapshot using the rules.
use super::{Card, authored::CardRules};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

thread_local! {
    static ACTIVE: RefCell<Option<Vec<(Card, &'static CardRules)>>> = const { RefCell::new(None) };
}

/// Thread-bound lifetime of one scenario's immutable executable fixtures.
pub struct Guard(PhantomData<Rc<()>>);

/// Install complete typed fixture rules on this thread until the returned guard
/// drops (including unwinding). Nested scopes and duplicate identities are errors.
pub fn activate(rows: &[(Card, &'static CardRules)]) -> Guard {
    for (index, (card, _)) in rows.iter().enumerate() {
        assert!(
            rows[..index].iter().all(|(prior, _)| prior != card),
            "duplicate fixture identity"
        );
    }
    let rows = rows.to_vec();
    ACTIVE.with(|active| {
        let mut active = active.borrow_mut();
        assert!(active.is_none(), "nested fixture rules");
        *active = Some(rows);
    });
    Guard(PhantomData)
}

impl Drop for Guard {
    fn drop(&mut self) {
        ACTIVE.with(|active| *active.borrow_mut() = None);
    }
}

pub(super) fn rules(card: Card) -> Option<&'static CardRules> {
    ACTIVE.with(|active| {
        active.borrow().as_ref().and_then(|rows| {
            rows.iter()
                .find(|(identity, _)| *identity == card)
                .map(|(_, rules)| *rules)
        })
    })
}
