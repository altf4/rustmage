pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    keywords: Keywords::FLYING.union(Keywords::TRAMPLE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: one(TargetRule::Player {
            opponent_only: true,
        }),
        program: &[
            Instruction::MayPayCostsFor {
                players: PlayerSet::Targeted,
                costs: &[ResolutionCost {
                    objects: Some(ResolutionObjectCost {
                        cost: AdditionalCost {
                            kind: EffectChoiceKind::Sacrifice,
                            filter: SacrificeFilter {
                                keywords: Keywords::NONE,
                                source_only: false,
                                kinds: 0,
                                nonland: true,
                                other: false,
                                tokens: Some(false),
                                artifact_enchantment_or_token: false,
                                subtype: None,
                            },
                        },
                        count: 2,
                    }),
                    ..ResolutionCost::FREE
                }],
                skip: 1,
            },
            Instruction::Skip(1),
            Instruction::Draw(PlayerSet::Controller, 2),
        ],
    }],
    ..CardRules::EMPTY
};
