pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPayCosts {
                    costs: &[ResolutionCost {
                        objects: Some(ResolutionObjectCost {
                            cost: AdditionalCost {
                                kind: EffectChoiceKind::Sacrifice,
                                filter: SacrificeFilter {
                                    keywords: Keywords::NONE,
                                    source_only: false,
                                    kinds: 1 | 32,
                                    nonland: false,
                                    other: true,
                                    tokens: None,
                                    artifact_enchantment_or_token: false,
                                    subtype: None,
                                },
                            },
                            count: 1,
                        }),
                        ..ResolutionCost::FREE
                    }],
                    skip: 1,
                },
                Instruction::Reflexive { ability: 1 },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: one(TargetRule::Any),
            program: &[Instruction::Damage(2)],
        },
    ],
    ..CardRules::EMPTY
};
