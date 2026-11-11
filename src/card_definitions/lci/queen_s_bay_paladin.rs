// The entry counter and target's graveyard mana value survive the zone change.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersOrAttacks,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Filtered {
                base: &TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 0,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                predicate: &TargetPredicate::Subtype("Vampire"),
            },
            min: 0,
            max: 1,
        }),
        program: &[
            Instruction::ReturnToBattlefieldWithCounter(CounterKind::Finality),
            Instruction::LoseLifeTargetCardManaValue(PlayerSet::Controller),
        ],
    }],
    ..CardRules::EMPTY
};
