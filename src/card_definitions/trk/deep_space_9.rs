// Complete pinned DeepSpace9.java and Oracle: each trigger moves one exact
// top-four batch to exile, then puts only its still-exiled permanents in play.
const ARRIVAL: &[Instruction] = &[
    Instruction::ExileTopLibraryRemember {
        players: PlayerSet::Controller,
        amount: AmountValue::Fixed(4),
    },
    Instruction::PutRememberedExiledPermanentsOntoBattlefield,
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: ARRIVAL,
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::Upkeep,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: ARRIVAL,
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::SourceCounterBonus {
            kind: CounterKind::Charge,
            minimum: 9,
            filter: CreatureFilter {
                relation: 0,
                other: false,
                subtype: None,
                keyword: Keywords::NONE,
                attacking: false,
                tokens: None,
                artifact: false,
            },
            stats: Stats {
                power: 3,
                toughness: 3,
            },
        }],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::TapPermanents(1),
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 1,
                    nonland: false,
                    other: true,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
            }),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCountersScaled {
            subject: Subject::Source,
            kind: CounterKind::Charge,
            amount: AmountValue::TappedCostCreaturePower,
        }],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        sorcery: true,
        once_per_turn: false,
        cycling: false,
        exhaust: false,
    }],
    ..CardRules::EMPTY
};
