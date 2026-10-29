// Direct Oracle and pinned XMage behavior; gated pending integration tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters, targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPayMana { cost: ManaCost { colored: [0; 6], generic: 2 }, skip: 1 },
                Instruction::Reflexive { ability: 1 },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Object { zone: TargetZone::Battlefield, kinds: 1 | 64,
                    relation: 1, other: false, nonland: false, tapped: false,
                    min_mana: 0, max_mana: u32::MAX },
                min: 0, max: 1,
            }),
            program: &[Instruction::Destroy],
        },
    ],
    activated: &[ActivatedAbility {
        exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield,
        cycling: false, once_per_turn: false, sorcery: true,
        cost: ActivationCost { colored: [0; 6], generic: 2, alternatives: &[], additional: None,
            mana_component: true, tap: false, sacrifice: false, exile_self: false,
            discard: false, life: 0, remove: None, loyalty: None },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield, kinds: 1, relation: 0, other: false,
            nonland: false, tapped: false, min_mana: 0, max_mana: u32::MAX,
        })),
        program: &[Instruction::AttachSource],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1, first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::AttachmentStats(Stats { power: 2, toughness: 1 })],
    }],
    extras: &CardExtras {
        equipment: Some(EquipmentSpec { stats: Stats { power: 2, toughness: 1 }, keywords: Keywords::NONE }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
