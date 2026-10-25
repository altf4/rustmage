// Complete frozen Oracle: choose an unused mode as the trigger is announced each turn.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 5),
    keywords: Keywords::FLYING.union(Keywords::VIGILANCE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Noncreature,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ModalOncePerTurn(&[
            SpellMode {
                text: "Solar Beam — The Vision gains double strike until end of turn.",
                targets: TargetPlan::NONE,
                program: &[Instruction::GrantKeywords {
                    subject: Subject::Source,
                    keywords: Keywords::DOUBLE_STRIKE,
                }],
            },
            SpellMode {
                text: "Density Control — The Vision gains indestructible until end of turn.",
                targets: TargetPlan::NONE,
                program: &[Instruction::GrantKeywords {
                    subject: Subject::Source,
                    keywords: Keywords::INDESTRUCTIBLE,
                }],
            },
            SpellMode {
                text: "Technopathy — Draw a card.",
                targets: TargetPlan::NONE,
                program: &[Instruction::Draw(PlayerSet::Controller, 1)],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
