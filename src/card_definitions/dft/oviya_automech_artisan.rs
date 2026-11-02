// Static defender relation and successful-entry artifact counters use exact result witnesses.
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::OpponentPlayerAttackerKeywords(
        Keywords::TRAMPLE,
    )],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 2),
    static_groups: GROUPS,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 1, 0],
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::CaptureEntryResult,
            Instruction::PutHandFiltered {
                filter: SearchFilter::AnyOf(&[
                    SearchFilter::Creature,
                    SearchFilter::Subtype("Vehicle"),
                ]),
                skip: 0,
            },
            Instruction::PutEntryResultCountersIfKinds {
                kinds: 16,
                kind: CounterKind::PlusOne,
                amount: 2,
            },
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
