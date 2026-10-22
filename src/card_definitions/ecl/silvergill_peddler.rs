// Direct typed behavior audited against complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfBecomesTapped,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::Discard(PlayerSet::Controller, 1),
        ],
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
    keywords: Keywords::NONE,
    ..CardRules::EMPTY
};
