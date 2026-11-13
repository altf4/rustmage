// Direct typed behavior from the pinned Oracle and XMage sources.
const BONUS: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 1,
        toughness: 1,
    },
    keywords: Keywords::FLYING,
};
const ATTACHMENT: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Ability,
    fragments: &[
        StaticAbilityFragment::AttachmentStats(BONUS.stats),
        StaticAbilityFragment::AttachmentKeywords(BONUS.keywords),
    ],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 0,
            max: 1,
        }),
        program: &[Instruction::AirbendTargets],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        sorcery: true,
        once_per_turn: false,
        targets: one(TargetRule::creature_you_control()),
        program: &[Instruction::AttachSource],
    }],
    static_groups: ATTACHMENT,
    extras: &CardExtras {
        equipment: Some(BONUS),
        static_ability_groups: ATTACHMENT,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
