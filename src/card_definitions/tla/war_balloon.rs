// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: Keywords::FLYING,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Type,
            fragments: &[StaticAbilityFragment::SourceCounterCreature {
                own_turn: false,
                kind: CounterKind::Fire,
                minimum: 3,
                form: SourceCreatureForm::ArtifactCreature,
                subtypes: &[],
                keywords: Keywords::NONE,
            }],
        },
    ],
    activated: &[
        ActivatedAbility {
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            exhaust: false,
            once_per_turn: false,
            sorcery: false,
            cost: ActivationCost {
                generic: 1,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::Fire,
                amount: 1,
            }],
        },
        ActivatedAbility {
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            exhaust: false,
            once_per_turn: false,
            sorcery: false,
            cost: ActivationCost::ZERO,
            targets: TargetPlan::NONE,
            program: &[Instruction::Crew { minimum: 3 }],
        },
    ],
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 4,
            toughness: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
