// Direct typed behavior from the pinned Oracle and complete XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: Keywords::FLASH,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: one(TargetRule::Creature),
            program: &[Instruction::May { skip: 1 }, Instruction::TapOrUntap],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfLeavesBattlefield,
            targets: one(TargetRule::Creature),
            program: &[Instruction::May { skip: 1 }, Instruction::TapOrUntap],
        },
    ],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 3,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: false,
            sacrifice: true,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 2)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
