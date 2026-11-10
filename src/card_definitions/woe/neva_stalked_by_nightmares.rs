// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::MENACE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::MENACE)],
    }],
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 1 | 32,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[Instruction::ReturnToHand],
        },
        TriggeredAbility {
            condition: TriggerCondition::PermanentToGraveyard {
                kinds: 32,
                relation: CastRelation::You,
                exclude_source_kinds: 0,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
                Instruction::Scry(1),
            ],
        },
    ],
    ..CardRules::EMPTY
};
