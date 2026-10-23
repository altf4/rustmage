// Exact post-mill recovery and a land-discard activation; no wording evaluation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    extras: &CardExtras {
        cost_alternatives: &[
            ManaCost {
                colored: [0, 0, 1, 0, 1, 0],
                generic: 0,
            },
            ManaCost {
                colored: [0, 0, 2, 0, 0, 0],
                generic: 0,
            },
        ],
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(4)),
            Instruction::SelectMilled(MilledSelectionSpec {
                filter: MilledFilter::Kinds(1 | 2),
                count: MilledCount::Range {
                    minimum: 0,
                    maximum: 1,
                },
                destination: MilledDestination::LibraryTop,
            }),
        ],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 3, 0],
            generic: 2,
            alternatives: &[
                ManaCost {
                    colored: [0, 0, 1, 0, 2, 0],
                    generic: 2,
                },
                ManaCost {
                    colored: [0, 0, 2, 0, 1, 0],
                    generic: 2,
                },
                ManaCost {
                    colored: [0, 0, 3, 0, 0, 0],
                    generic: 2,
                },
            ],
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::Discard,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 2,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
            }),
            mana_component: true,
            tap: true,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokensScaled {
            token: Card(5356),
            amount: AmountValue::Stat(StatValue::Count(CountFilter {
                zone: CountZone::Graveyard,
                relation: 0,
                kind: 2,
                exclude: 0,
                subtype: None,
                basic: false,
                other: false,
                cards_only: true,
            })),
            tapped: false,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
