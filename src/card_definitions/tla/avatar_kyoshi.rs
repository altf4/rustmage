// The back face of The Legend of Kyoshi.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledPermanentKeywords {
            kinds: 2,
            other: false,
            keywords: Keywords::TRAMPLE.union(Keywords::HEXPROOF),
        }],
    }],
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(
            ManaProduction::TapGreatestControlledCreaturePower { colors: 31 },
        )],
    }],
    ..CardRules::EMPTY
};
