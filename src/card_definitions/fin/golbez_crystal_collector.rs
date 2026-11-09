const ARTIFACTS: StatValue = StatValue::Count(CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 16,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: false,
});

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::ArtifactEntersYou,
            targets: TargetPlan::NONE,
            program: &[Instruction::Surveil(1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::Expression {
                    value: ARTIFACTS,
                    minimum: Some(4),
                    maximum: None,
                },
            },
            targets: one(TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            program: &[
                Instruction::ReturnToHand,
                Instruction::IfAmountAtLeast {
                    value: AmountValue::Stat(ARTIFACTS),
                    minimum: 8,
                    skip: 1,
                },
                Instruction::LoseLifeScaled(PlayerSet::Opponent, AmountValue::TargetCreaturePower),
            ],
        },
    ],
    ..CardRules::EMPTY
};
