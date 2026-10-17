// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 6),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 0, 0],
            generic: 0,
            alternatives: &[],
            additional: None,
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
        program: &[Instruction::AddMana(ManaProduction::Choice {
            mask: 31,
            amount: 1,
        })],
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
    keywords: Keywords::REACH.union(Keywords::VIGILANCE),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                Keywords::REACH.union(Keywords::VIGILANCE),
            )],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CombatRestriction(
                0,
                CombatRestriction {
                    subject: Subject::Source,
                    attack: true,
                    block: true,
                    unless: Some(StepGuard::Expression {
                        value: StatValue::Count(CountFilter {
                            zone: CountZone::Graveyard,
                            relation: 0,
                            kind: 0,
                            exclude: 0,
                            subtype: Some("Lesson"),
                            basic: false,
                            other: false,
                            cards_only: true,
                        }),
                        minimum: Some(3),
                        maximum: None,
                    }),
                },
            )],
        },
    ],
    extras: &CardExtras {
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                    Keywords::REACH.union(Keywords::VIGILANCE),
                )],
            },
            StaticAbilityGroup {
                line: 2,
                first_layer: AbilityLayer::Rule,
                fragments: &[StaticAbilityFragment::CombatRestriction(
                    0,
                    CombatRestriction {
                        subject: Subject::Source,
                        attack: true,
                        block: true,
                        unless: Some(StepGuard::Expression {
                            value: StatValue::Count(CountFilter {
                                zone: CountZone::Graveyard,
                                relation: 0,
                                kind: 0,
                                exclude: 0,
                                subtype: Some("Lesson"),
                                basic: false,
                                other: false,
                                cards_only: true,
                            }),
                            minimum: Some(3),
                            maximum: None,
                        }),
                    },
                )],
            },
        ],
        combat_restriction: Some(CombatRestriction {
            subject: Subject::Source,
            attack: true,
            block: true,
            unless: Some(StepGuard::Expression {
                value: StatValue::Count(CountFilter {
                    zone: CountZone::Graveyard,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: Some("Lesson"),
                    basic: false,
                    other: false,
                    cards_only: true,
                }),
                minimum: Some(3),
                maximum: None,
            }),
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
