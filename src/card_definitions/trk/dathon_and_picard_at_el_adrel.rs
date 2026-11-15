// Pinned Oracle and complete XMage class: the two modes have disjoint targets.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Temba, His Arms Wide",
            targets: one(TargetRule::Creature),
            program: &[
                Instruction::Grow {
                    subject: Subject::Targets,
                    power: 3,
                    toughness: 3,
                },
                Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords::TRAMPLE,
                },
            ],
        },
        SpellMode {
            text: "Shaka, When the Walls Fell",
            targets: TargetPlan::NONE,
            program: &[Instruction::ChooseObjects {
                kind: EffectChoiceKind::Sacrifice,
                players: PlayerSet::Opponent,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 16 | 32,
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
    ]),
    ..CardRules::EMPTY
};
