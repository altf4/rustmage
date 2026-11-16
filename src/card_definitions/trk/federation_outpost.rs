// Pinned XMage: entry-time untapped status chooses the modal trigger.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    entry: EntryTapped::UnlessAtLeastTwo,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersUntapped,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "You gain 2 life.",
                targets: TargetPlan::NONE,
                program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
            },
            SpellMode {
                text: "Surveil 1.",
                targets: TargetPlan::NONE,
                program: &[Instruction::Surveil(1)],
            },
        ])],
    }],
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
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            1, 0, 0, 0, 0, 0,
        ]))],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
