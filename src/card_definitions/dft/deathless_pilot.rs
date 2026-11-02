// Full pinned Oracle/XMage: contribution bonus is as-though power, not a stat boost.
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::IntrinsicKeywords(
        Keywords::CREW_SADDLE_PLUS_TWO,
    )],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::CREW_SADDLE_PLUS_TWO,
    static_groups: GROUPS,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Graveyard,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 3,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnSourceToHand(TargetZone::Graveyard)],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
