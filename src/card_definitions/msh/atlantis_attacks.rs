// Directly authored from the full pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Target player creates a 6/5 blue Leviathan creature token with hexproof.",
            targets: one(TargetRule::Player {
                opponent_only: false,
            }),
            program: &[Instruction::CreateTokensFor {
                players: PlayerSet::Targeted,
                token: Card::MshLeviathanToken,
                amount: 1,
                tapped: false,
            }],
        },
        SpellMode {
            text: "Return one or two target nonland permanents to their owners’ hands.",
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 0,
                    relation: -1,
                    other: false,
                    nonland: true,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                min: 1,
                max: 2,
            }),
            program: &[Instruction::ReturnToHand],
        },
        SpellMode {
            text: "Both, in printed order.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Player {
                    opponent_only: false,
                }),
                TargetSpec {
                    rule: TargetRule::Object {
                        zone: TargetZone::Battlefield,
                        kinds: 0,
                        relation: -1,
                        other: false,
                        nonland: true,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                    min: 1,
                    max: 2,
                },
            ]),
            program: &[
                Instruction::CreateTokensFor {
                    players: PlayerSet::Targeted,
                    token: Card::MshLeviathanToken,
                    amount: 1,
                    tapped: false,
                },
                Instruction::SelectGroup(1),
                Instruction::ReturnToHand,
            ],
        },
    ]),
    extras: &CardExtras {
        teamwork: Some(TeamworkSpec {
            minimum: 4,
            paid_targets: None,
            normal_modes: 3,
            paid_modes: 4,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
