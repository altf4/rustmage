// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OtherFilteredEntersYou(EntryPredicate::KindOrSubtype {
            kinds: 16,
            subtype: "Villain",
        }),
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        program: &[Instruction::Damage(1)],
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
    keywords: Keywords(0),
    ..CardRules::EMPTY
};
