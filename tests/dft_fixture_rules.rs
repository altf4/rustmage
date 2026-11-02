use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;

const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::HASTE,
    entry: EntryTapped::Always,
    extras: &CardExtras {
        exhaust_mask: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn immutable_fixture_rules_are_scoped_and_preserve_physical_metadata() {
    let card = Card::GrizzlyBears;
    let original_cost = card.cost();
    let original_name = card.name();
    let original_kind = card.kind_mask();
    let _scope = fixture_rules::activate(&[(card, &RULES)]);
    assert!(card.keywords().contains(Keywords::HASTE));
    assert_eq!(card.entry_tapped(), EntryTapped::Always);
    assert_eq!(card.exhaust_mask(), 1);
    assert_eq!(card.cost(), original_cost);
    assert_eq!(card.name(), original_name);
    assert_eq!(card.kind_mask(), original_kind);
}

#[test]
fn fixture_rules_drop_on_unwind_and_reject_nested_or_duplicate_scopes() {
    let card = Card::GrizzlyBears;
    let original = card.keywords();
    let result = std::panic::catch_unwind(|| {
        let _scope = fixture_rules::activate(&[(card, &RULES)]);
        assert!(std::panic::catch_unwind(|| fixture_rules::activate(&[(card, &RULES)])).is_err());
        assert!(card.keywords().contains(Keywords::HASTE));
        panic!("fixture unwind");
    });
    assert!(result.is_err());
    assert_eq!(card.keywords(), original);
    assert!(
        std::panic::catch_unwind(|| fixture_rules::activate(&[(card, &RULES), (card, &RULES)]))
            .is_err()
    );
    let _scope = fixture_rules::activate(&[(card, &RULES)]);
    assert!(card.keywords().contains(Keywords::HASTE));
}
