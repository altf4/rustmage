// Directly authored from the complete pinned Oracle and XMage implementation.
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLASH)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::FLASH,
    static_groups: GROUPS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "You gain 3 life and surveil 3.",
                targets: TargetPlan::NONE,
                program: &[
                    Instruction::GainLife(PlayerSet::Controller, 3),
                    Instruction::Surveil(3),
                ],
            },
            SpellMode {
                text: "Destroy target enchantment.",
                targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 32,
                    relation: -1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                })),
                program: &[Instruction::Destroy],
            },
            SpellMode {
                text: "Counter target creature spell.",
                targets: TargetPlan::Single(TargetSpec::one(TargetRule::SpellKinds {
                    kinds: 1,
                    excluded: 0,
                })),
                program: &[Instruction::Counter],
            },
        ])],
    }],
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
