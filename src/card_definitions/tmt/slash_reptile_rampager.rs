// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(7, 5),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::OtherCreatureEntersYou,
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::DamagePlayers(PlayerSet::Opponent, 2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::CreateTokens {
                token: Card(5245),
                amount: 1,
                tapped: false,
            }],
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
    keywords: Keywords(0),
    ..CardRules::EMPTY
};
