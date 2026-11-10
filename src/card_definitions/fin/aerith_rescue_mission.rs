pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Take the Elevator — Create three 1/1 colorless Hero creature tokens.",
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5328),
                amount: 3,
                tapped: false,
            }],
        },
        SpellMode {
            text: "Take 59 Flights of Stairs — Tap up to three target creatures. Put a stun counter on one of them.",
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 0,
                max: 3,
            }),
            program: &[
                Instruction::Tap,
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::CountersOnTarget {
                        group: 0,
                        kind: CounterKind::Stun,
                        amount: 1,
                    },
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
                    optional_skip: None,
                },
            ],
        },
    ]),
    ..CardRules::EMPTY
};
