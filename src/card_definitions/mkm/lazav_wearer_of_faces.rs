pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 0,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[
                Instruction::ExileGraveTargetsLinked,
                Instruction::CreateTokens {
                    token: Card(5296),
                    amount: 1,
                    tapped: false,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Sacrificed {
                own_turn: false,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: Some("Clue"),
                },
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::MayCopyLinkedExiledCreatureUntilEndOfTurn],
        },
    ],
    ..CardRules::EMPTY
};
