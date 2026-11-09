pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    entry: EntryTapped::Always,
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
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 0, 1, 0,
        ]))],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        adventure_identity: Some("Zanarkand, Ancient Metropolis // Lasting Fayth"),
        adventure_spell: Some(Card(5530)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
