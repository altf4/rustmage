// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfTransforms,
            targets: TargetPlan::NONE,
            program: &[Instruction::Search(SearchSpec {
                filter: SearchFilter::Land,
                maximum: 1,
                destination: SearchDestination::Battlefield,
                tapped: true,
                reveal: false,
            })],
        },
        TriggeredAbility {
            condition: TriggerCondition::LandEntersYou,
            targets: TargetPlan::NONE,
            program: &[Instruction::GrowPermanentGroup {
                filters: &[BattlefieldFilter {
                    kinds: 0,
                    relation: 0,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: Some("Bird"),
                    max_mana: u32::MAX,
                }],
                power: 1,
                toughness: 0,
            }],
        },
    ],
    ..CardRules::EMPTY
};
