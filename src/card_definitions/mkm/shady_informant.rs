pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: one(TargetRule::Any),
        program: &[Instruction::Damage(2)],
    }],
    extras: &CardExtras {
        disguise: Some(DisguiseSpec {
            mana: ManaCost {
                colored: [0, 0, 2, 0, 0, 0],
                generic: 2,
            },
            alternatives: &[
                ManaCost {
                    colored: [0, 0, 1, 1, 0, 0],
                    generic: 2,
                },
                ManaCost {
                    colored: [0, 0, 0, 2, 0, 0],
                    generic: 2,
                },
            ],
            modifier: DisguiseCostModifier::None,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
