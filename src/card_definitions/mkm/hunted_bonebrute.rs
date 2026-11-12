pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 2),
    keywords: Keywords::MENACE,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: one(TargetRule::Player {
                opponent_only: true,
            }),
            program: &[Instruction::CreateTokensFor {
                players: PlayerSet::Targeted,
                token: Card(5256),
                amount: 2,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDies,
            targets: TargetPlan::NONE,
            program: &[Instruction::LoseLife(PlayerSet::Opponent, 3)],
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::MENACE)],
    }],
    extras: &CardExtras {
        disguise_cost: Some(ManaCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 1,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
