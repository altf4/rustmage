// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Surveil(2),
            Instruction::GainLife(PlayerSet::Controller, 2),
            Instruction::IfCastFromGraveyard { skip: 1 },
            Instruction::ExileSourceReturnTransformed {
                counter: CounterKind::Finality,
            },
        ],
    ),
    extras: &CardExtras {
        transforming_double_face: true,
        flashback_cost: Some(ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
