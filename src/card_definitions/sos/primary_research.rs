// Direct typed Oracle/XMage behavior; full printed obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: one(TargetRule::Filtered {
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
            }),
            program: &[Instruction::ReturnToBattlefield],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::Expression {
                    value: StatValue::CardLeftGraveyardThisTurn,
                    minimum: Some(1),
                    maximum: None,
                },
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
    ],
    ..CardRules::EMPTY
};
