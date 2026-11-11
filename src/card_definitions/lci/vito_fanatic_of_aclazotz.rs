pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Sacrificed {
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 0,
                nonland: false,
                other: true,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
            own_turn: false,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::IfThisAbilityResolutionNumber { number: 1, skip: 1 },
            Instruction::GainLife(PlayerSet::Controller, 2),
            Instruction::IfThisAbilityResolutionNumber { number: 2, skip: 1 },
            Instruction::LoseLife(PlayerSet::Opponent, 2),
            Instruction::IfThisAbilityResolutionNumber { number: 3, skip: 1 },
            Instruction::CreateTokens {
                token: Card(5578),
                amount: 1,
                tapped: false,
            },
        ],
    }],
    ..CardRules::EMPTY
};
