// Complete Oracle/XMage: live controlled Zombie lord and exact acquired graveyard embalm.
const BONUS: StaticModifier = StaticModifier {
    own_turn: false,
    requires_life_gain: false,
    filter: CreatureFilter {
        relation: 0,
        other: false,
        subtype: Some("Zombie"),
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    },
    stats: Stats {
        power: 1,
        toughness: 1,
    },
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Modify,
    fragments: &[StaticAbilityFragment::StaticModifier(0, BONUS)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: GROUPS,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[Instruction::GrantEmbalmTargets],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        static_modifiers: &[BONUS],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
