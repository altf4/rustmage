pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    entry: EntryTapped::Always,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 5,
            mana_component: true,
            tap: true,
            exile_self: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ShuffleGraveyardIntoLibraryCount { excluded_kinds: 2 },
            Instruction::GainLifeMoved,
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
