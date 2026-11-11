const EQUIPMENT: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 1,
        toughness: 0,
    },
    keywords: Keywords::NONE,
};
const GRANT: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[
        StaticAbilityFragment::AttachmentStats(EQUIPMENT.stats),
        StaticAbilityFragment::AttachmentTrigger(0),
    ],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: GRANT,
    extras: &CardExtras {
        equipment: Some(EQUIPMENT),
        static_ability_groups: GRANT,
        ..CardExtras::EMPTY
    },
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::GrantedAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::MaySacrificeForGrantedReflexive {
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: true,
                    kinds: 0,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                ability: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: one(TargetRule::Any),
            program: &[Instruction::Damage(2)],
        },
    ],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::creature_you_control()),
        program: &[Instruction::AttachSource],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
