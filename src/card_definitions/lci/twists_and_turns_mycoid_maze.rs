const SCRY_BEFORE_EXPLORE: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::ScryBeforeControlledCreatureExplores],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: SCRY_BEFORE_EXPLORE,
    extras: &CardExtras {
        static_ability_groups: SCRY_BEFORE_EXPLORE,
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: one(TargetRule::creature_you_control()),
            program: &[Instruction::Explore(Subject::Targets)],
        },
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                event: &TriggerCondition::LandEntersYou,
                guard: StepGuard::Expression {
                    value: StatValue::Count(CountFilter {
                        zone: CountZone::Battlefield,
                        relation: 0,
                        kind: 2,
                        exclude: 0,
                        subtype: None,
                        basic: false,
                        other: false,
                        cards_only: false,
                    }),
                    minimum: Some(7),
                    maximum: None,
                },
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::TransformSource],
        },
    ],
    ..CardRules::EMPTY
};
