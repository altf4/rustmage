// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Each opponent sacrifices a nontoken artifact of their choice.",
            targets: TargetPlan::NONE,
            program: &[Instruction::ChooseObjects {
                kind: EffectChoiceKind::Sacrifice,
                players: PlayerSet::Opponent,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 16,
                    nonland: false,
                    other: false,
                    tokens: Some(false),
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 1,
                optional_skip: None,
            }],
        },
        SpellMode {
            text: "Return target artifact or creature card with mana value 2 or less from your graveyard to the battlefield.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 17,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: 2,
            })),
            program: &[Instruction::ReturnToBattlefield],
        },
        SpellMode {
            text: "Creatures you control get +1/+1 and gain trample until end of turn.",
            targets: TargetPlan::NONE,
            program: &[
                Instruction::GrowCreatures {
                    filter: CreatureFilter {
                        relation: 0,
                        other: false,
                        subtype: None,
                        keyword: Keywords::NONE,
                        attacking: false,
                        tokens: None,
                        artifact: false,
                    },
                    power: 1,
                    toughness: 1,
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
                    keywords: Keywords::TRAMPLE,
                },
            ],
        },
    ]),
    ..CardRules::EMPTY
};
