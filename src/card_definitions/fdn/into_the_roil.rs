// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 0,
            relation: -1,
            other: false,
            nonland: true,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[
            Instruction::ReturnToHand,
            Instruction::IfKicked { skip: 1 },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
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
        kicker_cost: Some(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 1,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
