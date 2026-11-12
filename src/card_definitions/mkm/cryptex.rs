pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[
        ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::CollectEvidence(3),
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
                }),
                ..ActivationCost::ZERO
            },
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            sorcery: false,
            once_per_turn: false,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::AddMana(ManaProduction::Choice {
                    mask: 31,
                    amount: 1,
                }),
                Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::Unlock,
                    amount: 1,
                },
            ],
        },
        ActivatedAbility {
            cost: ActivationCost {
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            exhaust: false,
            guard: ActivationGuard::SourceCounterAtLeast {
                kind: CounterKind::Unlock,
                minimum: 5,
            },
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            sorcery: false,
            once_per_turn: false,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Surveil(3),
                Instruction::Draw(PlayerSet::Controller, 3),
            ],
        },
    ],
    ..CardRules::EMPTY
};
