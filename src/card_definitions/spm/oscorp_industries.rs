pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    entry: EntryTapped::Always,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersFromGraveyard,
        targets: TargetPlan::NONE,
        program: &[Instruction::LoseLife(PlayerSet::Controller, 2)],
    }],
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Choice {
            mask: 2 | 4 | 8,
            amount: 1,
        })],
    }],
    extras: &CardExtras {
        mayhem_land: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
