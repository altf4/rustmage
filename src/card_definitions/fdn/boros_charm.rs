// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Boros Charm deals 4 damage to target player or planeswalker.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Union(&[
                TargetRule::Player {
                    opponent_only: false,
                },
                TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 64,
                    relation: -1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: 4294967295,
                },
            ]))),
            program: &[Instruction::Damage(4)],
        },
        SpellMode {
            text: "Permanents you control gain indestructible until end of turn.",
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::GrantPermanentsThisTurn(Keywords(8192))],
        },
        SpellMode {
            text: "Target creature gains double strike until end of turn.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords(1024),
            }],
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
    keywords: Keywords(0),
    ..CardRules::EMPTY
};
