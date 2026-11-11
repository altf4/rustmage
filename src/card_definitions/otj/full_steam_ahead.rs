pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::GrowPlayerCreatures {
                players: PlayerSet::Controller,
                power: 2,
                toughness: 2,
            },
            Instruction::GrantCreatureKeywordAbilities {
                filter: CreatureFilter {
                    relation: 0,
                    other: false,
                    subtype: None,
                    keyword: Keywords::NONE,
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
                keywords: Keywords::TRAMPLE.union(Keywords::MAX_ONE_BLOCKER),
            },
        ],
    ),
    ..CardRules::EMPTY
};
