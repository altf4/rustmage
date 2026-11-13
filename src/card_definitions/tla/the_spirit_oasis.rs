// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::DrawScaled(
                PlayerSet::Controller,
                AmountValue::Stat(StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: Some("Shrine"),
                    basic: false,
                    other: false,
                    cards_only: false,
                })),
            )],
        },
        TriggeredAbility {
            condition: TriggerCondition::OtherFilteredEntersYou(EntryPredicate::SubtypeAny(&[
                "Shrine",
            ])),
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
    ],
    ..CardRules::EMPTY
};
