// Directly authored from complete pinned Oracle and XMage; gated pending validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PermanentEntersYou {
            kinds: 1,
            nontoken: false,
            subtype: None,
            include_source: true,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Surveil(1)],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::ScriedOrSurveilledThisTurn,
        origin: ActivationOrigin::Graveyard,
        cycling: false,
        targets: TargetPlan::NONE,
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            colored: [1, 1, 0, 0, 0, 0],
            generic: 0,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        program: &[Instruction::ReturnSourceToBattlefieldWithCounter {
            tapped: false,
            counter: CounterKind::Finality,
        }],
    }],
    ..CardRules::EMPTY
};
