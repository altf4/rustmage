// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::FLYING,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 0,
                    relation: -1,
                    other: true,
                    nonland: true,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                min: 0,
                max: 1,
            }),
            program: &[Instruction::AirbendTargets],
        },
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Subtype("Lesson"),
                ordinal: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GrantKeywords {
                subject: Subject::Source,
                keywords: Keywords::LIFELINK,
            }],
        },
    ],
    ..CardRules::EMPTY
};
