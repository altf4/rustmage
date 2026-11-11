// Direct typed back face from the paired Oracle and XMage sources.
const PERMANENT_CARDS: StatValue = StatValue::Count(CountFilter {
    zone: CountZone::Graveyard,
    relation: 0,
    kind: 1 | 2 | 16 | 32 | 64 | 256,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: true,
});
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
        program: &[Instruction::AddMana(ManaProduction::ChoiceStat {
            colors: 31,
            amount: &PERMANENT_CARDS,
        })],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
