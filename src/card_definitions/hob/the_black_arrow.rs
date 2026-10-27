// Full Oracle/XMage: actual damage qualification survives replacement choices.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: Keywords::FLASH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::Any),
        program: &[
            Instruction::DamageForSubtypeDestruction {
                amount: 1,
                subtype: "Dragon",
            },
            Instruction::DestroyDamagedSubtype,
        ],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        targets: one(TargetRule::creature_you_control()),
        program: &[Instruction::AttachSource],
        sorcery: true,
        once_per_turn: false,
        exhaust: false,
        cycling: false,
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::AttachmentStats(Stats {
                power: 1,
                toughness: 1,
            }),
            StaticAbilityFragment::AttachmentKeywords(Keywords::REACH),
        ],
    }],
    extras: &CardExtras {
        equipment: Some(EquipmentSpec {
            stats: Stats {
                power: 1,
                toughness: 1,
            },
            keywords: Keywords::REACH,
        }),
        static_ability_groups: &[StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[
                StaticAbilityFragment::AttachmentStats(Stats {
                    power: 1,
                    toughness: 1,
                }),
                StaticAbilityFragment::AttachmentKeywords(Keywords::REACH),
            ],
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
