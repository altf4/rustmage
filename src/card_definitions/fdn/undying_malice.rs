// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::GrantUntilEndTurn { ability: 0 }],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Granted(&TriggerCondition::SelfDies),
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::ReturnGrantedSource {
            counter: Some((CounterKind::PlusOne, 1)),
            role: None,
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
    keywords: Keywords(0),
    ..CardRules::EMPTY
};
