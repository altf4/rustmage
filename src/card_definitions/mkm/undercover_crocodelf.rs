pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Source,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card::from_catalogue_id(5296),
            amount: 1,
            tapped: false,
        }],
    }],
    extras: &CardExtras {
        disguise: Some(DisguiseSpec {
            mana: ManaCost {
                colored: [0, 0, 0, 0, 2, 0],
                generic: 3,
            },
            alternatives: &[
                ManaCost {
                    colored: [0, 1, 0, 0, 1, 0],
                    generic: 3,
                },
                ManaCost {
                    colored: [0, 2, 0, 0, 0, 0],
                    generic: 3,
                },
            ],
            modifier: DisguiseCostModifier::None,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
