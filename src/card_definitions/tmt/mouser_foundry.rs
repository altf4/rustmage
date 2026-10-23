// Direct typed behavior from the pinned Oracle and complete XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5277),
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfLeavesBattlefield,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5277),
                amount: 1,
                tapped: false,
            }],
        },
    ],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 4,
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
        targets: one(TargetRule::Creature),
        program: &[Instruction::Damage(3)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
