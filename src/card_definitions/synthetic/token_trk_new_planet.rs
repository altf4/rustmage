// Printed Star Trek token behavior; source pinned in the paired provenance record.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Choice {
            mask: 31,
            amount: 1,
        })],
        sorcery: false,
        once_per_turn: false,
    }],
    deck_limit: None,
    ..CardRules::EMPTY
};
