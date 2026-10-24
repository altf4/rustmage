// Directly authored from the full pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Destroy target noncreature artifact.",
            targets: one(TargetRule::Filtered {
                base: &TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 16,
                    relation: -1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                predicate: &TargetPredicate::WithoutKinds(1),
            }),
            program: &[Instruction::Destroy],
        },
        SpellMode {
            text: "Target creature you control deals damage equal to its power to target creature an opponent controls.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                }),
                TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    relation: 1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                }),
            ]),
            program: &[Instruction::PowerDamage {
                source: 0,
                recipient: 1,
            }],
        },
        SpellMode {
            text: "Both, in printed order.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Filtered {
                    base: &TargetRule::Object {
                        zone: TargetZone::Battlefield,
                        kinds: 16,
                        relation: -1,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                    predicate: &TargetPredicate::WithoutKinds(1),
                }),
                TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                }),
                TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    relation: 1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                }),
            ]),
            program: &[
                Instruction::Destroy,
                Instruction::PowerDamage {
                    source: 1,
                    recipient: 2,
                },
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
