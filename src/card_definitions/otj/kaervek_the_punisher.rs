pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CommittedCrime,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Filtered {
                base: &TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 0,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                predicate: &TargetPredicate::Colors(1 << 2),
            },
            min: 0,
            max: 1,
        }),
        program: &[
            Instruction::ExileTargetCopyCastPaid,
            Instruction::IfOfferedCastDeclined { skip: 1 },
            Instruction::LoseLife(PlayerSet::Controller, 2),
        ],
    }],
    ..CardRules::EMPTY
};
