// Direct typed behavior from the pinned Oracle and complete XMage sources.
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup { line: 0, first_layer: AbilityLayer::Ability, fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLASH)] },
    StaticAbilityGroup { line: 2, first_layer: AbilityLayer::Modify, fragments: &[StaticAbilityFragment::AttachmentStats(Stats { power: 1, toughness: 2 })] },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: Keywords::FLASH,
    static_groups: GROUPS,
    triggers: &[TriggeredAbility { condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::creature_you_control()),
        program: &[Instruction::AttachSource, Instruction::GrantKeywords { subject: Subject::Targets, keywords: Keywords::WITHER }],
    }],
    activated: &[ActivatedAbility {
        exhaust: false, guard: ActivationGuard::Always, cycling: false, origin: ActivationOrigin::Battlefield,
        cost: ActivationCost { colored: [0;6], generic: 2, alternatives: &[], additional: None, mana_component: true, tap: false, sacrifice: false, exile_self: false, discard: false, life: 0, remove: None, loyalty: None },
        targets: one(TargetRule::creature_you_control()), program: &[Instruction::AttachSource],
        sorcery: true, once_per_turn: false,
    }],
    extras: &CardExtras { equipment: Some(EquipmentSpec { stats: Stats { power: 1, toughness: 2 }, keywords: Keywords::NONE }), static_ability_groups: GROUPS, ..CardExtras::EMPTY },
    ..CardRules::EMPTY
};
