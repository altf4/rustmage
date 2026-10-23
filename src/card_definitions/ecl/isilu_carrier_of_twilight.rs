const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 2,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::ControlledCreatureTrigger {
        ability: 0,
        other: true,
        nontoken: true,
    }],
}];
// Persist is granted to each other nontoken creature; death uses departure LKI.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: Keywords::FLYING.union(Keywords::LIFELINK),
    extras: &CardExtras {
        transform: Some(TransformSpec {
            front: Card(1273),
            back: Card(5350),
            front_name: "Eirdu, Carrier of Dawn",
        }),
        ..CardExtras::EMPTY
    },
    deck_limit: None,
    static_groups: GROUPS,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Granted(&TriggerCondition::ReturnDeath(
                ReturnDeathFilter::SourceWithoutCounter(CounterKind::MinusOne),
            )),
            targets: TargetPlan::NONE,
            program: &[Instruction::ReturnDeathCard(ReturnKind::Persist)],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::Main1,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPayMana {
                    cost: ManaCost {
                        colored: [1, 0, 0, 0, 0, 0],
                        generic: 0,
                    },
                    skip: 1,
                },
                Instruction::TransformSource,
            ],
        },
    ],
    ..CardRules::EMPTY
};
