pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 1),
    keywords: Keywords::VIGILANCE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfTurnedFaceUp,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card::from_catalogue_id(5256),
            amount: 2,
            tapped: true,
        }],
    }],
    extras: &CardExtras {
        disguise: Some(DisguiseSpec {
            mana: ManaCost {
                colored: [0, 0, 0, 2, 0, 0],
                generic: 0,
            },
            alternatives: &[
                ManaCost {
                    colored: [1, 0, 0, 1, 0, 0],
                    generic: 0,
                },
                ManaCost {
                    colored: [2, 0, 0, 0, 0, 0],
                    generic: 0,
                },
            ],
            modifier: DisguiseCostModifier::None,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
