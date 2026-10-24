// Direct typed Oracle/XMage behavior; full printed obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::creature_you_control()),
        &[Instruction::BlinkImmediately {
            counter: Some((CounterKind::PlusOne, 1)),
        }],
    ),
    extras: &CardExtras {
        flashback_cost: Some(ManaCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 2,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
