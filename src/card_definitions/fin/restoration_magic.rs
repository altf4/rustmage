// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Cure — {0} — Target permanent gains hexproof and indestructible until end of turn.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 0,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE),
            }],
        },
        SpellMode {
            text: "Cura — {1} — Target permanent gains hexproof and indestructible until end of turn. You gain 3 life.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 0,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[
                Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE),
                },
                Instruction::GainLife(PlayerSet::Controller, 3),
            ],
        },
        SpellMode {
            text: "Curaga — {3}{W} — Permanents you control gain hexproof and indestructible until end of turn. You gain 6 life.",
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[
                Instruction::GrantPermanentsThisTurn(
                    Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE),
                ),
                Instruction::GainLife(PlayerSet::Controller, 6),
            ],
        },
    ]),
    triggers: &[],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords::NONE,
    extras: &CardExtras {
        tiered_costs: Some(&[
            ManaCost {
                colored: [0, 0, 0, 0, 0, 0],
                generic: 0,
            },
            ManaCost {
                colored: [0, 0, 0, 0, 0, 0],
                generic: 1,
            },
            ManaCost {
                colored: [1, 0, 0, 0, 0, 0],
                generic: 3,
            },
        ]),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
