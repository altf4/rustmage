// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            generic: 2,
            tap: true,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ModalOnce(&[
            SpellMode {
                text: "Deal 2 damage to target creature",
                targets: one(TargetRule::Creature),
                program: &[Instruction::Damage(2)],
            },
            SpellMode {
                text: "Tap target creature",
                targets: one(TargetRule::Creature),
                program: &[Instruction::Tap],
            },
            SpellMode {
                text: "Sacrifice this artifact and gain 3 life",
                targets: TargetPlan::NONE,
                program: &[
                    Instruction::SacrificeSource,
                    Instruction::GainLife(PlayerSet::Controller, 3),
                ],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
