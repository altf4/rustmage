pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            exile_self: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::AddMana(ManaProduction::Fixed([0, 1, 0, 0, 0, 0])),
            Instruction::CopyNextInstantSorcery { ability: 0 },
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyCapturedSpell {
            count: SpellCopyCount::One,
            retarget: true,
        }],
    }],
    ..CardRules::EMPTY
};
