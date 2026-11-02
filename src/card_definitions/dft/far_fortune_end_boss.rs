// Max-speed replacement applies to each controlled source's damage, combat or otherwise.
const BONUS: ReplacementSpec = ReplacementSpec::ConditionalControlledDamageToOpponentPlus {
    condition: StatValue::AtLeast {
        value: &StatValue::ControllerSpeed,
        minimum: 4,
    },
    amount: 1,
};
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::StartYourEngines],
    },
    StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, BONUS)],
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 5),
    static_groups: GROUPS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttackDeclared {
            minimum: 1,
            power_min: None,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::DamagePlayers(PlayerSet::Opponent, 1)],
    }],
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        replacement_effects: &[BONUS],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
