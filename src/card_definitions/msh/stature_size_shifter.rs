// Whole printed behavior; see paired frozen Oracle and complete Java provenance.
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::CannotBeBlockedIfSourcePowerAtMost(1)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 2, 0, 0, 0, 0],
            generic: 0,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ActivationX,
            Instruction::PutCountersScaled {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: AmountValue::ChosenX,
            },
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    static_groups: GROUPS,
    extras: &CardExtras {
        power_up_abilities: &[0],
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
