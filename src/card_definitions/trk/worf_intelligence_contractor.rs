// Oracle and official TRK release notes reviewed; the pinned XMage class is absent.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: 3,
            },
            min: 0,
            max: 1,
        }),
        program: &[
            Instruction::GainControlUntilEndTurn,
            Instruction::Untap,
            Instruction::ChangeCreatureTypesUntilEndTurn {
                subject: Subject::Targets,
                types: &["Warrior"],
                additional: true,
            },
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::HASTE,
            },
        ],
    }],
    ..CardRules::EMPTY
};
