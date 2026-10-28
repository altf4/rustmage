// Direct Oracle behavior with staged additional entry counters and flashback.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: 3,
        })),
        &[Instruction::ReturnToBattlefieldWithCounter(
            CounterKind::PlusOne,
        )],
    ),
    extras: &CardExtras {
        flashback_cost: Some(ManaCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 4,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
