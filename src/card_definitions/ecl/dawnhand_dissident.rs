// Direct typed behavior from the complete pinned Oracle and XMage sources.
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 2,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::LinkedExileCreatureCounterCasting(3)],
}];
const fn blight_tap(amount: u32) -> ActivationCost {
    ActivationCost {
        colored: [0; 6],
        generic: 0,
        alternatives: &[],
        mana_component: false,
        additional: Some(AdditionalCost {
            kind: EffectChoiceKind::BlightCost(amount),
            filter: SacrificeFilter {
                kinds: 1,
                keywords: Keywords::NONE,
                source_only: false,
                nonland: false,
                other: false,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
        }),
        tap: true,
        sacrifice: false,
        exile_self: false,
        discard: false,
        life: 0,
        remove: None,
        loyalty: None,
    }
}
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 2),
    activated: &[
        ActivatedAbility {
            cost: blight_tap(1),
            targets: TargetPlan::NONE,
            program: &[Instruction::Surveil(1)],
            sorcery: false,
            once_per_turn: false,
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
        },
        ActivatedAbility {
            cost: blight_tap(2),
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 0,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[Instruction::ExileTargetsLinkedToSource],
            sorcery: false,
            once_per_turn: false,
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
        },
    ],
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
