// Complete Oracle body authored from the paired pinned XMage source.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::AttachmentKeywords(Keywords::NONE)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::AttachmentStats(Stats {
                power: 0,
                toughness: 0,
            })],
        },
    ],
    extras: &CardExtras {
        equipment: Some(EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        }),
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttachedDealtCombatDamage,
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::Charge,
            amount: 1,
        }],
    }],
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                colored: [0; 6],
                generic: 1,
                alternatives: &[],
                additional: None,
                mana_component: true,
                tap: false,
                sacrifice: false,
                exile_self: false,
                discard: false,
                life: 0,
                remove: None,
                loyalty: None,
            },
            targets: one(TargetRule::creature_you_control()),
            program: &[Instruction::AttachSource],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                colored: [0; 6],
                generic: 0,
                alternatives: &[],
                additional: None,
                mana_component: false,
                tap: false,
                sacrifice: false,
                exile_self: false,
                discard: false,
                life: 0,
                remove: Some((Some(CounterKind::Charge), 1)),
                loyalty: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Modal(&[
                SpellMode {
                    text: "Untap target land.",
                    targets: one(TargetRule::Object {
                        zone: TargetZone::Battlefield,
                        kinds: 2,
                        relation: -1,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    }),
                    program: &[Instruction::Untap],
                },
                SpellMode {
                    text: "Target creature can't block this turn.",
                    targets: one(TargetRule::Creature),
                    program: &[Instruction::GrantKeywords {
                        subject: Subject::Targets,
                        keywords: Keywords::CANT_BLOCK,
                    }],
                },
                SpellMode {
                    text: "Put a +1/+1 counter on equipped creature.",
                    targets: TargetPlan::NONE,
                    program: &[Instruction::PutCounters {
                        subject: Subject::Attached,
                        kind: CounterKind::PlusOne,
                        amount: 1,
                    }],
                },
            ])],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
