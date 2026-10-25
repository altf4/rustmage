// Direct typed behavior for all three frozen Oracle chapters.
const ARTIFACTS: CountFilter = CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    cards_only: false,
    kind: 16,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::May { skip: 2 },
                Instruction::DrawScaled(
                    PlayerSet::Controller,
                    AmountValue::Stat(StatValue::Count(ARTIFACTS)),
                ),
                Instruction::Draw(PlayerSet::Opponent, 1),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: &[Instruction::ReduceArtifactSpellCostsThisTurn(1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: one(TargetRule::Player {
                opponent_only: true,
            }),
            program: &[Instruction::DamageScaled(AmountValue::Stat(
                StatValue::GreatestManaValue(ARTIFACTS),
            ))],
        },
    ],
    ..CardRules::EMPTY
};
