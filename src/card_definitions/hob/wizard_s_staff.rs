// Complete pinned Oracle, WizardsStaff.java and reusable prowess/equip helpers.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::AttachmentTrigger(0)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::AdditionalTriggers {
                scope: AdditionalTriggerScope::EquippedCreature,
                condition: StatValue::Fixed(1),
            }],
        },
    ],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Granted(&TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Noncreature,
            ordinal: None,
        }),
        targets: TargetPlan::NONE,
        program: &[Instruction::Grow {
            subject: Subject::Source,
            power: 1,
            toughness: 1,
        }],
    }],
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Battlefield,
            cost: ActivationCost {
                colored: [0, 0, 0, 0, 0, 0],
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
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
                base: &TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                predicate: &TargetPredicate::Subtype("Wizard"),
            })),
            program: &[Instruction::AttachSource],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Battlefield,
            cost: ActivationCost {
                colored: [0, 0, 0, 0, 0, 0],
                generic: 3,
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
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[Instruction::AttachSource],
            sorcery: true,
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
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
