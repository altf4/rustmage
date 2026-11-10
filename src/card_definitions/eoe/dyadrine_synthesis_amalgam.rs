pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 1),
    keywords: Keywords::TRAMPLE,
    extras: &CardExtras {
        x_symbols: 1,
        entry_counter_rules: &[EntryCounterRule {
            guard: StepGuard::Always,
            amount: EntryCounterAmount::ManaSpent,
        }],
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttackDeclared {
            minimum: 1,
            power_min: None,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayCosts {
                costs: &[ResolutionCost {
                    objects: Some(ResolutionObjectCost {
                        cost: AdditionalCost {
                            kind: EffectChoiceKind::RemoveCreatureCounter(CounterKind::PlusOne),
                            filter: SacrificeFilter {
                                kinds: 1,
                                other: false,
                                keywords: Keywords::NONE,
                                source_only: false,
                                nonland: false,
                                tokens: None,
                                artifact_enchantment_or_token: false,
                                subtype: None,
                            },
                        },
                        count: 2,
                    }),
                    ..ResolutionCost::FREE
                }],
                skip: 2,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::CreateTokens {
                token: Card(5309),
                amount: 1,
                tapped: false,
            },
        ],
    }],
    ..CardRules::EMPTY
};
