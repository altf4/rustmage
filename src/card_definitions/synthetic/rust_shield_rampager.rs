// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfOffspring,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::CreateTokens {
            token: Card(5250),
            amount: 1,
            tapped: false,
        }],
    }],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords::NO_POWER_LE_TWO_BLOCKERS,
    deck_limit: None,
    extras: &CardExtras {
        offspring_cost: Some(ManaCost {
            colored: [0, 0, 0, 0, 0, 0],
            generic: 2,
        }),
        offspring_copy: Some(Card(5250)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
