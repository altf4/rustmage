// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InZone {
            zone: TargetZone::Graveyard,
            event: &TriggerCondition::CardDrawn {
                relation: CastRelation::You,
                ordinal: Some(2),
            },
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
            Instruction::ReturnSourceToBattlefieldWithCounter {
                tapped: false,
                counter: CounterKind::Finality,
            },
        ],
    }],
    ..CardRules::EMPTY
};
