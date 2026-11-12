pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    entry: EntryTapped::Always,
    keywords: Keywords::HEXPROOF,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::ChosenColor)],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::Color),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
