// Directly authored from the complete pinned Oracle and both XMage modes.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Creatures target opponent controls get -1/-1 until end of turn.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
                opponent_only: true,
            })),
            program: &[Instruction::GrowPlayerCreatures {
                players: PlayerSet::Targeted,
                power: -1,
                toughness: -1,
            }],
        },
        SpellMode {
            text: "Return up to two target creature cards from your graveyard to your hand.",
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 1,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                min: 0,
                max: 2,
            }),
            program: &[Instruction::ReturnToHand],
        },
    ]),
    ..CardRules::EMPTY
};
