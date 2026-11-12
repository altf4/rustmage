pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersOrTurnsFaceUp,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card::from_catalogue_id(5266),
            amount: 1,
            tapped: false,
        }],
    }],
    extras: &CardExtras {
        disguise: Some(DisguiseSpec {
            mana: ManaCost {
                colored: [0, 2, 0, 0, 0, 0],
                generic: 0,
            },
            alternatives: &[
                ManaCost {
                    colored: [0, 1, 0, 1, 0, 0],
                    generic: 0,
                },
                ManaCost {
                    colored: [0, 0, 0, 2, 0, 0],
                    generic: 0,
                },
            ],
            modifier: DisguiseCostModifier::None,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
