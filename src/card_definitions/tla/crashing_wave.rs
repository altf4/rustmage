// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 0,
            max: usize::MAX,
        }),
        &[
            Instruction::Tap,
            Instruction::DistributeCountersAmongPermanents {
                total: 3,
                kind: CounterKind::Stun,
                filter: BattlefieldFilter {
                    kinds: 1,
                    relation: 1,
                    nonland: false,
                    tapped: Some(true),
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                },
            },
        ],
    ),
    extras: &CardExtras {
        waterbend_additional: Some(WaterbendAdditional::RequiredX),
        targets_up_to_x: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
