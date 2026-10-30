// Direct Oracle and pinned XMage behavior; gated pending interaction validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    entry: EntryTapped::UnlessBehold("Jace"),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        targets: TargetPlan::NONE,
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 2,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: true,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        program: &[Instruction::EnsureJaceToken, Instruction::EmpowerJace(2)],
    }],
    ..CardRules::EMPTY
};
