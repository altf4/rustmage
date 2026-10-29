// Pinned FRA Lotus token; gated pending integration and rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false, guard: ActivationGuard::Always, cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost { colored: [0; 6], generic: 0, alternatives: &[], additional: None, mana_component: false, tap: true, sacrifice: true, exile_self: false, discard: false, life: 0, remove: None, loyalty: None },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Choice { mask: 31, amount: 3 })],
        sorcery: false, once_per_turn: false,
    }],
    deck_limit: None,
    ..CardRules::EMPTY
};
