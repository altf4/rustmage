// Resolution samples controller speed even after the source is sacrificed as a cost.
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::StartYourEngines],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: GROUPS,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            generic: 3,
            colored: [0, 0, 0, 0, 1, 0],
            mana_component: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::SearchScaled {
            spec: SearchSpec {
                filter: SearchFilter::BasicLand,
                maximum: 0,
                destination: SearchDestination::Battlefield,
                tapped: true,
                reveal: false,
            },
            maximum: AmountValue::Stat(StatValue::ControllerSpeed),
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
