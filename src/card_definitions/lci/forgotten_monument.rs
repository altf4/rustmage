const TAP_COLORLESS: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::AddMana(ManaProduction::Fixed([
        0, 0, 0, 0, 0, 1,
    ]))],
    sorcery: false,
    once_per_turn: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[
        TAP_COLORLESS,
        ActivatedAbility {
            cost: ActivationCost {
                life: 1,
                ..TAP_COLORLESS.cost
            },
            program: &[Instruction::AddMana(ManaProduction::any_color(1))],
            ..TAP_COLORLESS
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::OtherControlledSubtypeTapManaActivation {
                subtype: "Cave",
                ability: 1,
            },
        ],
    }],
    ..CardRules::EMPTY
};
