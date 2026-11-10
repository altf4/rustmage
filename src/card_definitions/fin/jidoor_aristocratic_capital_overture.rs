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
            0, 1, 0, 0, 0, 0,
        ]))],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        adventure_identity: Some("Jidoor, Aristocratic Capital // Overture"),
        adventure_spell: Some(Card(5528)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
