const OPPONENT_ARTIFACTS_ENTER_TAPPED: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::OpponentEntryTapped(0, 16)],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 4),
    static_groups: OPPONENT_ARTIFACTS_ENTER_TAPPED,
    extras: &CardExtras {
        opponent_entry_tapped: 16,
        static_ability_groups: OPPONENT_ARTIFACTS_ENTER_TAPPED,
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [1, 0, 0, 0, 0, 0],
            mana_component: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ActivationXWithMinimum {
                symbols: 2,
                minimum: 0,
            },
            Instruction::DestroyManaValueExactly {
                kinds: 16,
                amount: AmountValue::ChosenX,
            },
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
