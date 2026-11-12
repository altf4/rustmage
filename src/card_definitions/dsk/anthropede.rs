// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    keywords: Keywords::REACH,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::REACH)],
    }],
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPayCosts {
                    costs: &[
                        ResolutionCost {
                            objects: Some(ResolutionObjectCost {
                                cost: AdditionalCost {
                                    kind: EffectChoiceKind::Discard,
                                    filter: SacrificeFilter {
                                        keywords: Keywords::NONE,
                                        source_only: false,
                                        kinds: 0,
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
                        },
                        ResolutionCost {
                            mana: generic(2),
                            ..ResolutionCost::FREE
                        },
                    ],
                    skip: 1,
                },
                Instruction::Reflexive { ability: 1 },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: one(TargetRule::Filtered {
                base: &object(0),
                predicate: &TargetPredicate::Subtype("Room"),
            }),
            program: &[Instruction::Destroy],
        },
    ],
    ..CardRules::EMPTY
};
