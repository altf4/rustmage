// Pinned Oracle and full XMage class: exile, then offer the former controller a basic land.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Union(&[
            TargetRule::Creature,
            TargetRule::Filtered {
                base: &TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 0,
                    relation: -1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                predicate: &TargetPredicate::Subtype("Spacecraft"),
            },
        ])),
        &[
            Instruction::Exile,
            Instruction::MayTargetController { skip: 1 },
            Instruction::SearchTargetController(SearchSpec {
                filter: SearchFilter::BasicLand,
                maximum: 1,
                destination: SearchDestination::Hand,
                tapped: false,
                reveal: true,
            }),
        ],
    ),
    ..CardRules::EMPTY
};
