// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfOffspring,
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::CreateTokens {
                token: Card(5280),
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDies,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
                opponent_only: false,
            })),
            program: &[Instruction::Damage(1)],
        },
    ],
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
    deck_limit: None,
    extras: &CardExtras {
        offspring_cost: Some(ManaCost {
            colored: [0, 0, 0, 0, 0, 0],
            generic: 2,
        }),
        offspring_copy: Some(Card(5280)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
