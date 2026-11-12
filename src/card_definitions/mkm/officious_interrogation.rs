pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Player {
                opponent_only: false,
            },
            min: 0,
            max: 2,
        }),
        &[Instruction::CreateTokensRepeated {
            token: Card::from_catalogue_id(5296),
            amount: AmountValue::TargetPlayersCreatureCount,
        }],
    ),
    extras: &CardExtras {
        additional_target_mana: Some(ManaCost {
            colored: [1, 1, 0, 0, 0, 0],
            generic: 0,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
