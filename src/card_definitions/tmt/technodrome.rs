// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 0, 0],
            generic: 0,
            alternatives: &[],
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::Sacrifice,
                filter: SacrificeFilter {
                    artifact_enchantment_or_token: false,
                    keywords: Keywords(0),
                    source_only: false,
                    kinds: 16,
                    nonland: false,
                    other: true,
                    tokens: None,
                    subtype: None,
                },
            }),
            mana_component: false,
            tap: true,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords(258),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(258))],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CombatRestriction(
                0,
                CombatRestriction {
                    subject: Subject::Source,
                    attack: true,
                    block: true,
                    unless: Some(StepGuard::SourcePowerAtLeast(6)),
                },
            )],
        },
    ],
    extras: &CardExtras {
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(258))],
            },
            StaticAbilityGroup {
                line: 1,
                first_layer: AbilityLayer::Rule,
                fragments: &[StaticAbilityFragment::CombatRestriction(
                    0,
                    CombatRestriction {
                        subject: Subject::Source,
                        attack: true,
                        block: true,
                        unless: Some(StepGuard::SourcePowerAtLeast(6)),
                    },
                )],
            },
        ],
        combat_restriction: Some(CombatRestriction {
            subject: Subject::Source,
            attack: true,
            block: true,
            unless: Some(StepGuard::SourcePowerAtLeast(6)),
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
