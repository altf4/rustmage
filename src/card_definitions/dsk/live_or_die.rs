// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Return target creature card from your graveyard to the battlefield.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[Instruction::ReturnToBattlefield],
        },
        SpellMode {
            text: "Destroy target creature.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
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
