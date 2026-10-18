// Direct typed behavior from complete Oracle and pinned XMage implementations.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::CardNameAfterOpponentHand),
        named_activation_lock: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
