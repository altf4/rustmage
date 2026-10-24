// Complete typed behavior; select from exact first public destinations of this mill.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 3),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0; 6],
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
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(4)),
            Instruction::SelectMilled(MilledSelectionSpec {
                filter: MilledFilter::KindsOrSubtype {
                    kinds: 32,
                    subtype: "Hero",
                },
                count: MilledCount::Range {
                    minimum: 0,
                    maximum: 1,
                },
                destination: MilledDestination::Hand,
            }),
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
