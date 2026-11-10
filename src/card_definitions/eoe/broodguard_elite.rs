// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfLeaves,
        targets: one(TargetRule::creature_you_control()),
        program: &[Instruction::PutSourceCountersOnTargets],
    }],
    extras: &CardExtras {
        x_symbols: 1,
        x_entry_counters: true,
        warp: Some(&WarpSpec {
            x_symbols: 1,
            ..WarpSpec::fixed(ManaCost {
                generic: 0,
                colored: [0, 0, 0, 0, 1, 0],
            })
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
