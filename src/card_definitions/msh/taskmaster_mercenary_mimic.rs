// Full frozen Oracle/official notes. No direct Taskmaster Java class exists in the pinned tree.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Main1,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Union(&[
                TargetRule::Creature,
                TargetRule::Filtered {
                    base: &TargetRule::Object {
                        zone: TargetZone::Graveyard,
                        kinds: 1,
                        relation: -1,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                    predicate: &TargetPredicate::Nontoken,
                },
            ]),
            min: 0,
            max: 1,
        }),
        program: &[Instruction::BecomeCopy(InPlaceCopySpec {
            recipient_group: None,
            blueprint_group: 0,
            overrides: crate::engine::CopyOverrides {
                name: Some("Taskmaster, Mercenary Mimic"),
                legendary: Some(true),
                kinds: Some(1),
                types: Some(crate::engine::CopyTypes {
                    replace: true,
                    human_villain: true,
                    mercenary: true,
                    nightmare: false,
                    golem: false,
                    fractal: false,
                }),
                keywords: Keywords::NONE,
            },
            add_kinds: 0,
            base: None,
            next_turn: true,
        })],
    }],
    ..CardRules::EMPTY
};
