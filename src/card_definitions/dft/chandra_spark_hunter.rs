// Complete Oracle and pinned XMage implementation.
const fn loyalty(amount: i32) -> ActivationCost {
    ActivationCost {
        loyalty: Some(amount),
        ..ActivationCost::ZERO
    }
}
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        starting_loyalty: 4,
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Filtered {
                base: &TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 0,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                predicate: &TargetPredicate::Subtype("Vehicle"),
            },
            min: 0,
            max: 1,
        }),
        program: &[Instruction::AnimateTargets(AnimationSpec {
            artifact: true,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: None,
            keywords: Keywords::HASTE,
            duration: AnimationDuration::EndOfTurn,
        })],
    }],
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: loyalty(2),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::SacrificeOrDiscard { nonland: false },
                    players: PlayerSet::Controller,
                    filter: SacrificeFilter {
                        keywords: Keywords::NONE,
                        source_only: false,
                        kinds: 16,
                        nonland: false,
                        other: false,
                        tokens: None,
                        artifact_enchantment_or_token: false,
                        subtype: None,
                    },
                    amount: 1,
                    optional_skip: Some(1),
                },
                Instruction::Draw(PlayerSet::Controller, 1),
            ],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: loyalty(0),
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5470),
                amount: 1,
                tapped: false,
            }],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: loyalty(-7),
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateEmblem(Card(5471))],
            sorcery: true,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
