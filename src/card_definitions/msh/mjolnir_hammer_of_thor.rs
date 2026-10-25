// Full frozen Oracle and pinned Java. Worthiness restricts equip targets, not other attachments.
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(
        0,
        ReplacementSpec::DoubleEquippedDamage,
    )],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: GROUPS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 0,
            max: 1,
        }),
        program: &[Instruction::Damage(4)],
    }],
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Battlefield,
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
            targets: one(TargetRule::Filtered {
                base: &TargetRule::Filtered {
                    base: &TargetRule::Filtered {
                        base: &TargetRule::creature_you_control(),
                        predicate: &TargetPredicate::Legendary,
                    },
                    predicate: &TargetPredicate::WithoutSubtype("Villain"),
                },
                predicate: &TargetPredicate::Colors(1 | 8),
            }),
            program: &[Instruction::AttachSource],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Hand,
            cost: ActivationCost {
                colored: [0, 0, 0, 1, 0, 0],
                generic: 2,
                alternatives: &[],
                additional: None,
                mana_component: true,
                tap: false,
                sacrifice: false,
                exile_self: false,
                discard: true,
                life: 0,
                remove: None,
                loyalty: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::DamageBattlefield {
                filters: &[BattlefieldFilter {
                    kinds: 1,
                    relation: -1,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                }],
                players: None,
                amount: 2,
            }],
            sorcery: false,
            once_per_turn: false,
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
        replacement_effects: &[ReplacementSpec::DoubleEquippedDamage],
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
