// Oracle and official Star Trek release notes; pinned XMage class is absent.
const OWN_OTHER_CREATURE: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1,
    relation: 0,
    other: true,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::MaySkipOwnUntap],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: one(OWN_OTHER_CREATURE),
        program: &[Instruction::DoubleTargetTriggersWhileSourceTapped],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
