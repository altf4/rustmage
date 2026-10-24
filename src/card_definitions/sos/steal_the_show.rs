// Whole-card typed Oracle behavior; full XMage source pinned beside this file.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Discard any number, then draw that many.",
            targets: one(TargetRule::Player {
                opponent_only: false,
            }),
            program: &[Instruction::DiscardAnyDraw {
                players: PlayerSet::Targeted,
                bonus: 0,
                required_all: false,
            }],
        },
        SpellMode {
            text: "Deal graveyard instant and sorcery count damage.",
            targets: one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 65,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            program: &[Instruction::DamageScaled(AmountValue::Stat(
                StatValue::Count(CountFilter {
                    zone: CountZone::Graveyard,
                    relation: 0,
                    kind: 12,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: true,
                }),
            ))],
        },
        SpellMode {
            text: "Both, in printed order.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Player {
                    opponent_only: false,
                }),
                TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 65,
                    relation: -1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                }),
            ]),
            program: &[
                Instruction::DiscardAnyDraw {
                    players: PlayerSet::Targeted,
                    bonus: 0,
                    required_all: false,
                },
                Instruction::SelectGroup(1),
                Instruction::DamageScaled(AmountValue::Stat(StatValue::Count(CountFilter {
                    zone: CountZone::Graveyard,
                    relation: 0,
                    kind: 12,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: true,
                }))),
            ],
        },
    ]),
    ..CardRules::EMPTY
};
