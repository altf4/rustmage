// Source-audited direct typed definition; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec {
                rule: TargetRule::Filtered {
                    base: &TargetRule::Object {
                        zone: TargetZone::Graveyard,
                        kinds: 0,
                        relation: 0,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                    predicate: &TargetPredicate::Permanent,
                },
                min: 0,
                max: 2,
            },
            TargetSpec::one(TargetRule::Player {
                opponent_only: false,
            }),
            TargetSpec {
                rule: TargetRule::Grouped {
                    base: &TargetRule::Object {
                        zone: TargetZone::Graveyard,
                        kinds: 0,
                        relation: -1,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                    constraint: TargetGroupConstraint::GraveyardOwnerOfPlayerGroup(1),
                },
                min: 0,
                max: 4,
            },
        ]),
        &[
            Instruction::SelectGroup(0),
            Instruction::ReturnToHand,
            Instruction::ShuffleGraveyardTargetsIntoPlayerLibrary {
                player_group: 1,
                cards_group: 2,
            },
            Instruction::ExileResolvingSpell,
        ],
    ),

    ..CardRules::EMPTY
};
