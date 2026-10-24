// Directly authored from the full pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Exile target creature with toughness 4 or greater.",
            targets: one(TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::ToughnessAtLeast(4),
            }),
            program: &[Instruction::Exile],
        },
        SpellMode {
            text: "Exile target enchantment with mana value 4 or greater.",
            targets: one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 32,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 4,
                max_mana: u32::MAX,
            }),
            program: &[Instruction::Exile],
        },
        SpellMode {
            text: "Both, in printed order.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Filtered {
                    base: &TargetRule::Creature,
                    predicate: &TargetPredicate::ToughnessAtLeast(4),
                }),
                TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 32,
                    relation: -1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 4,
                    max_mana: u32::MAX,
                }),
            ]),
            program: &[
                Instruction::Exile,
                Instruction::SelectGroup(1),
                Instruction::Exile,
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
