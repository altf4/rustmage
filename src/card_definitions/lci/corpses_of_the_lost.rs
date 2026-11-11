const SKELETON_PIRATE: Card = Card::from_catalogue_id(5577);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::ControlledSubtypeKeywords {
                subtype: "Skeleton",
                keywords: Keywords::HASTE,
            },
            StaticAbilityFragment::CreatureGroupBonus {
                filters: &[CreatureFilter {
                    relation: 0,
                    other: false,
                    subtype: Some("Skeleton"),
                    keyword: Keywords::NONE,
                    attacking: false,
                    tokens: None,
                    artifact: false,
                }],
                stats: Stats {
                    power: 1,
                    toughness: 0,
                },
            },
        ],
    }],
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: SKELETON_PIRATE,
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::Expression {
                    value: StatValue::DescendedThisTurn,
                    minimum: Some(1),
                    maximum: None,
                },
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPayControllerLife { amount: 1, skip: 2 },
                Instruction::LoseLife(PlayerSet::Controller, 1),
                Instruction::ReturnSourceToHand(TargetZone::Battlefield),
            ],
        },
    ],
    ..CardRules::EMPTY
};
