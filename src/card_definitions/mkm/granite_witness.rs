pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::FLYING.union(Keywords::VIGILANCE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfTurnedFaceUp,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        program: &[Instruction::May { skip: 1 }, Instruction::TapOrUntap],
    }],
    extras: &CardExtras {
        disguise: Some(DisguiseSpec {
            mana: ManaCost {
                colored: [2, 0, 0, 0, 0, 0],
                generic: 0,
            },
            alternatives: &[
                ManaCost {
                    colored: [1, 1, 0, 0, 0, 0],
                    generic: 0,
                },
                ManaCost {
                    colored: [0, 2, 0, 0, 0, 0],
                    generic: 0,
                },
            ],
            modifier: DisguiseCostModifier::None,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
