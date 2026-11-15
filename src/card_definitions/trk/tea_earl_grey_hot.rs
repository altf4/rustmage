// Oracle and pinned XMage reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            tap: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "This artifact deals 2 damage to target opponent.",
                targets: one(TargetRule::Player { opponent_only: true }),
                program: &[Instruction::Damage(2)],
            },
            SpellMode {
                text: "You gain 3 life.",
                targets: TargetPlan::NONE,
                program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
