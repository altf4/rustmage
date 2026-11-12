// Directly authored from the paired Oracle and pinned XMage sources.
const OTHER_ENCHANTMENT_OR_CREATURE: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 1 | 32,
    nonland: false,
    other: true,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
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
                                filter: OTHER_ENCHANTMENT_OR_CREATURE,
                            },
                            count: 1,
                        }),
                        ..ResolutionCost::FREE
                    }],
                    skip: 1,
                },
                Instruction::Draw(PlayerSet::Controller, 2),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Sacrificed {
                filter: SacrificeFilter {
                    kinds: 0,
                    other: false,
                    source_only: true,
                    ..OTHER_ENCHANTMENT_OR_CREATURE
                },
                own_turn: false,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::ManifestDread],
        },
    ],
    ..CardRules::EMPTY
};
