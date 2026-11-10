pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    entry: EntryTapped::Always,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayCosts {
                costs: &[ResolutionCost {
                    objects: Some(ResolutionObjectCost {
                        cost: AdditionalCost {
                            kind: EffectChoiceKind::TapPermanents(1),
                            filter: SacrificeFilter {
                                kinds: 0,
                                other: false,
                                keywords: Keywords::NONE,
                                source_only: false,
                                nonland: false,
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
            Instruction::Skip(1),
            Instruction::SacrificeSource,
        ],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::any_color(1))],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        exhaust: false,
        cycling: false,
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
