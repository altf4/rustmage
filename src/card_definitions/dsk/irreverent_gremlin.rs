pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::MENACE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OtherFilteredEntersYou(EntryPredicate::CreaturePowerAtMost(2)),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::IfOptionalAbilityUnused { skip: 3 },
            Instruction::MayPayCosts {
                costs: &[ResolutionCost {
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
                }],
                skip: 2,
            },
            Instruction::RecordOncePerTurn,
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};
