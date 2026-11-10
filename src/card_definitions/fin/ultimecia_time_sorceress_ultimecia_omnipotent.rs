// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 5),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersOrAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::Surveil(2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPayCosts {
                    costs: &[ResolutionCost {
                        mana: ManaCost {
                            colored: [0, 2, 2, 0, 0, 0],
                            generic: 4,
                        },
                        objects: Some(ResolutionObjectCost {
                            cost: AdditionalCost {
                                kind: EffectChoiceKind::ExileGraveyardCards(8),
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
                            count: 8,
                        }),
                        ..ResolutionCost::FREE
                    }],
                    skip: 1,
                },
                Instruction::TransformSource,
            ],
        },
    ],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
