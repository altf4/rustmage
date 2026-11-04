// Source-audited direct typed definition; see paired provenance.
const GRAVE: CountFilter = CountFilter {
    zone: CountZone::Graveyard,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: true,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::HASTE,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::DiscardUpToDraw { maximum: 2 }],
        },
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                event: &TriggerCondition::SelfAttacks,
                guard: StepGuard::Expression {
                    value: StatValue::Count(GRAVE),
                    minimum: Some(7),
                    maximum: None,
                },
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::ExileRandomGraveyardPlayThisTurn],
        },
    ],
    ..CardRules::EMPTY
};
