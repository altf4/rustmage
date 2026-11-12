// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 16 | 32,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 0,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                min: 0,
                max: 4,
            },
        ]),
        &[
            Instruction::ShuffleTargetIntoLibrary,
            Instruction::MayPlayer {
                player: PlayerSet::Controller,
                skip: 1,
            },
            Instruction::ShuffleGraveyardTargetsIntoControllerLibrary { cards_group: 1 },
        ],
    ),
    ..CardRules::EMPTY
};
