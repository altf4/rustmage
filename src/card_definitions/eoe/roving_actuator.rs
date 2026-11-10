// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::VoidActive,
            event: &TriggerCondition::SelfEnters,
        },
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 4 | 8,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: 2,
            },
            min: 0,
            max: 1,
        }),
        program: &[Instruction::ExileTargetsCopyFreeWithBudget {
            mana_value: u64::MAX,
        }],
    }],
    ..CardRules::EMPTY
};
