// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::GrowScaled {
            subject: Subject::Targets,
            amount: AmountValue::ChosenX,
            power: -1,
            toughness: -1,
        }],
    ),
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
    extras: &CardExtras {
        x_symbols: 1,
        uncounterable: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
