pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(7, 5),
    extras: &CardExtras {
        warp: Some(&WarpSpec::fixed(ManaCost {
            generic: 4,
            ..ManaCost::ZERO
        })),
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfLeaves,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::PutHandPermanent {
                maximum: AmountValue::Stat(StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 2,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: false,
                })),
                tapped: true,
            },
        ],
    }],
    ..CardRules::EMPTY
};
