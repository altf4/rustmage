const KEYWORDS: Keywords = Keywords::FLYING.union(Keywords::VIGILANCE);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: KEYWORDS,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(KEYWORDS)],
    }],
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::ControlledSubtypeAttacks {
                subtype: "Bat",
                exclude: false,
                one_or_more: false,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersOrAttacks,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPayCosts {
                    costs: &[ResolutionCost {
                        mana: ManaCost {
                            colored: [1, 0, 1, 0, 0, 0],
                            generic: 0,
                        },
                        life: Some(AmountValue::Fixed(2)),
                        ..ResolutionCost::FREE
                    }],
                    skip: 1,
                },
                Instruction::Reflexive { ability: 2 },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
                base: &TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 0,
                    relation: 0,
                    other: false,
                    nonland: true,
                    tapped: false,
                    min_mana: 0,
                    max_mana: 3,
                },
                predicate: &TargetPredicate::Permanent,
            })),
            program: &[Instruction::ReturnToBattlefieldWithCounter(
                CounterKind::Finality,
            )],
        },
    ],
    ..CardRules::EMPTY
};
