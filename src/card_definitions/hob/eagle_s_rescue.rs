// Complete Oracle and pinned XMage behavior; the power restriction is targeting only.
const BONUS: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 2,
        toughness: 2,
    },
    keywords: Keywords::FLYING,
};
const AURA: AuraSpec = AuraSpec {
    kinds: 1,
    relation: -1,
    bonus: BONUS,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Ability,
    fragments: &[
        StaticAbilityFragment::AttachmentKeywords(Keywords::FLYING),
        StaticAbilityFragment::AttachmentStats(Stats {
            power: 2,
            toughness: 2,
        }),
    ],
}];
const HYBRID: &[ManaCost] = &[
    ManaCost {
        colored: [1, 1, 0, 0, 0, 0],
        generic: 2,
    },
    ManaCost {
        colored: [2, 0, 0, 0, 0, 0],
        generic: 2,
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::Creature)),
    aura: Some(AURA),
    static_groups: GROUPS,
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Graveyard,
        sorcery: true,
        once_per_turn: false,
        exhaust: false,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 2, 0, 0, 0, 0],
            generic: 2,
            alternatives: HYBRID,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::Filtered {
            base: &TargetRule::creature_you_control(),
            predicate: &TargetPredicate::PowerAtMost(1),
        }),
        program: &[Instruction::ReturnSourceAuraAttached],
    }],
    extras: &CardExtras {
        aura: Some(AURA),
        static_ability_groups: GROUPS,
        cost_alternatives: HYBRID,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
