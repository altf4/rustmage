// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: -1,
                other: true,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 0,
            max: 1,
        }),
        program: &[
            Instruction::IfSubjectControlledBy {
                subject: Subject::Targets,
                controller: PlayerSet::Controller,
                skip: 3,
            },
            Instruction::ReturnToHand,
            Instruction::CreateTokens {
                token: Card::from_catalogue_id(5296),
                amount: 1,
                tapped: false,
            },
            Instruction::Skip(1),
            Instruction::ReturnToHand,
        ],
    }],
    ..CardRules::EMPTY
};
