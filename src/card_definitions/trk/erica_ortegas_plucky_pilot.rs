// Oracle and official Star Trek release notes; pinned XMage class is absent.
const OWN_LAND: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 2,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::HASTE,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfBecomesTapped,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::CaptureEntryResult,
                Instruction::PutHandLand { tapped: false },
                Instruction::IfAmountAtLeast {
                    value: AmountValue::CapturedEntryResultCount,
                    minimum: 1,
                    skip: 1,
                },
                Instruction::ScheduleStep {
                    step: TriggerStep::End,
                    relation: CastRelation::Any,
                    ability: 1,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[Instruction::ChooseControlledPermanentToHand(OWN_LAND)],
        },
    ],
    ..CardRules::EMPTY
};
