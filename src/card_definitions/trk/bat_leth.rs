// Pinned Oracle and complete XMage class: targeted entry shrink and equip {2}.
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Modify,
    fragments: &[StaticAbilityFragment::AttachmentStats(Stats {
        power: 2,
        toughness: 0,
    })],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: 1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[Instruction::Grow {
            subject: Subject::Targets,
            power: -2,
            toughness: -2,
        }],
    }],
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
        sorcery: true,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::creature_you_control()),
        program: &[Instruction::AttachSource],
    }],
    static_groups: GROUPS,
    extras: &CardExtras {
        equipment: Some(EquipmentSpec {
            stats: Stats {
                power: 2,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        }),
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
