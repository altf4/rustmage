// Oracle and official Star Trek release notes; pinned XMage class is absent.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 4),
    keywords: Keywords::MENACE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InZone {
            zone: TargetZone::Graveyard,
            event: &TriggerCondition::FaceDilemma(CastRelation::You),
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayMana {
                cost: ManaCost {
                    colored: [0, 0, 1, 0, 0, 0],
                    generic: 0,
                },
                skip: 1,
            },
            Instruction::ReturnSourceToHand(TargetZone::Graveyard),
        ],
    }],
    ..CardRules::EMPTY
};
