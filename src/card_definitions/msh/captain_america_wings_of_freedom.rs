// Direct typed implementation: Oracle includes every other Hero permanent.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 1),
    keywords: Keywords::FLYING.union(Keywords::FIRST_STRIKE),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterUnlessPay {
                cost: ManaCost {
                    colored: [0; 6],
                    generic: 1,
                },
                subject: CounterSubject::Related,
                exile: false,
                unpaid_skip: 0,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::GrowPermanentGroupScaled {
                filters: &[BattlefieldFilter {
                    kinds: 0,
                    relation: 0,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: Some("Hero"),
                    max_mana: u32::MAX,
                }],
                exclude_source: true,
                amount: AmountValue::SourceToughness,
            }],
        },
    ],
    ..CardRules::EMPTY
};
