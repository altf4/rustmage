// Directly authored from pinned Oracle and XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::Discard(PlayerSet::Controller, 1),
        ],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        targets: TargetPlan::NONE,
        once_per_turn: false,
        sorcery: true,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 5,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: true,
            sacrifice: false,
            exile_self: true,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        program: &[Instruction::MoveBattlefield {
            filters: &[BattlefieldFilter {
                kinds: 1,
                relation: -1,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            exile: false,
        }],
    }],
    ..CardRules::EMPTY
};
