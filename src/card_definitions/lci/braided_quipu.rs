// Authored from pinned Oracle and XMage sources.
const ARTIFACTS: CountFilter = CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 16,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: false,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 3,
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
        targets: TargetPlan::NONE,
        program: &[
            Instruction::DrawScaled(
                PlayerSet::Controller,
                AmountValue::Stat(StatValue::Count(ARTIFACTS)),
            ),
            Instruction::PutSourceThirdFromTop,
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
