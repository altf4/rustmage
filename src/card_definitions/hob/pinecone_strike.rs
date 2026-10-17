// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Pinecone Strike deals 3 damage to target creature. If that creature would die this turn, exile it instead.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[Instruction::Damage(3), Instruction::ExileIfDiesThisTurn],
        },
        SpellMode {
            text: "Destroy target artifact token.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
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
                predicate: &TargetPredicate::Token,
            })),
            program: &[Instruction::Destroy],
        },
        SpellMode {
            text: "Choose both",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Creature),
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
                    predicate: &TargetPredicate::Token,
                }),
            ]),
            program: &[
                Instruction::Damage(3),
                Instruction::ExileIfDiesThisTurn,
                Instruction::SelectGroup(1),
                Instruction::Destroy,
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
    keywords: Keywords(0),
    ..CardRules::EMPTY
};
