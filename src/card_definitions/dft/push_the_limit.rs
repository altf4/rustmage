// Complete Oracle/XMage: successful exact entrants form the delayed sacrifice group.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ReturnGraveyardWithDelayedSacrifice(SearchFilter::AnyOf(&[
                SearchFilter::Subtype("Mount"),
                SearchFilter::Subtype("Vehicle"),
            ])),
            Instruction::AnimatePermanentGroup {
                filters: &[BattlefieldFilter {
                    kinds: 0,
                    relation: 0,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: Some("Vehicle"),
                    max_mana: u32::MAX,
                }],
                spec: AnimationSpec {
                    artifact: true,
                    creature_types: AnimationCreatureTypes::Unchanged,
                    base: None,
                    keywords: Keywords::NONE,
                    duration: AnimationDuration::EndOfTurn,
                },
            },
            Instruction::GrantCreatures {
                filter: CreatureFilter {
                    relation: 0,
                    other: false,
                    subtype: None,
                    keyword: Keywords::NONE,
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
                keywords: Keywords::HASTE,
            },
        ],
    ),
    ..CardRules::EMPTY
};
