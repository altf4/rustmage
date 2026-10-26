// Direct full Oracle translation; owner draws even when another player activates.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Ward,
        targets: TargetPlan::NONE,
        program: &[Instruction::CounterUnlessPay {
            cost: generic(3),
            subject: CounterSubject::Related,
            exile: false,
            unpaid_skip: 0,
        }],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 6,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ShuffleSourceIntoOwnerLibrary,
            Instruction::Draw(PlayerSet::SourceOwner, 3),
        ],
        exhaust: false,
        cycling: false,
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
