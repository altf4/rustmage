// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            program: &[Instruction::PermanentsDealFixedDamage {
                filters: &[BattlefieldFilter {
                    kinds: 1,
                    relation: 0,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                }],
                exclude_source: false,
                amount: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                event: &TriggerCondition::AtStep {
                    step: TriggerStep::End,
                    relation: CastRelation::You,
                    guard: StepGuard::Always,
                },
                guard: StepGuard::SourceUnsolved(&StepGuard::Expression {
                    value: StatValue::DistinctDeclaredAttackersThisTurn,
                    minimum: Some(3),
                    maximum: None,
                }),
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::SolveSource],
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::ControlledCreatureKindBonus {
            kinds: 1,
            stats: Stats {
                power: 1,
                toughness: 0,
            },
        }],
    }],
    extras: &CardExtras {
        solved_static_groups: &[2],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
