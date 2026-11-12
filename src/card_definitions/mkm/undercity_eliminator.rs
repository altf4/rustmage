pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
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
                                    kinds: 1 | 16,
                                    nonland: false,
                                    other: false,
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
            targets: one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            program: &[Instruction::Exile],
        },
    ],
    ..CardRules::EMPTY
};
