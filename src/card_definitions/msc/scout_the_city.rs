// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Look Around — Mill three cards. You may put a permanent card from among them into your hand. You gain 3 life.",
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[
                Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(3)),
                Instruction::RecoverMilled(RecoveryFilter::Permanent),
                Instruction::GainLife(PlayerSet::Controller, 3),
            ],
        },
        SpellMode {
            text: "Bring Down — Destroy target creature with flying.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::Keyword(Keywords(1)),
            })),
            program: &[Instruction::Destroy],
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
