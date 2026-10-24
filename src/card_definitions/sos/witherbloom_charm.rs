// Complete typed Oracle behavior; see paired source provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "You may sacrifice a permanent. If you do, draw two cards.",
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::Sacrifice,
                    players: PlayerSet::Controller,
                    filter: SacrificeFilter {
                        keywords: Keywords::NONE,
                        source_only: false,
                        kinds: 0,
                        nonland: false,
                        other: false,
                        tokens: None,
                        artifact_enchantment_or_token: false,
                        subtype: None,
                    },
                    amount: 1,
                    optional_skip: Some(1),
                },
                Instruction::Draw(PlayerSet::Controller, 2),
            ],
        },
        SpellMode {
            text: "You gain 5 life.",
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 5)],
        },
        SpellMode {
            text: "Destroy target nonland permanent with mana value 2 or less.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 0,
                relation: -1,
                other: false,
                nonland: true,
                tapped: false,
                min_mana: 0,
                max_mana: 2,
            })),
            program: &[Instruction::Destroy],
        },
    ]),
    ..CardRules::EMPTY
};
