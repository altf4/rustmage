// Complete Oracle behavior, checked against pinned XMage or official release notes.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "You gain 3 life.",
                targets: TargetPlan::NONE,
                program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
            },
            SpellMode {
                text: "Surveil 2.",
                targets: TargetPlan::NONE,
                program: &[Instruction::Surveil(2)],
            },
        ])],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            discard: true,
            ..ActivationCost::ZERO
        },
        origin: ActivationOrigin::Hand,
        guard: ActivationGuard::Always,
        targets: TargetPlan::NONE,
        program: &[Instruction::Search(SearchSpec {
            filter: SearchFilter::BasicLand,
            maximum: 1,
            destination: SearchDestination::Hand,
            tapped: false,
            reveal: true,
        })],
        sorcery: false,
        once_per_turn: false,
        cycling: true,
        exhaust: false,
    }],
    ..CardRules::EMPTY
};
