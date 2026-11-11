const NONLAND: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: true,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(7, 7),
    keywords: Keywords::MENACE
        .union(Keywords::TRAMPLE)
        .union(Keywords::LIFELINK)
        .union(Keywords::HEXPROOF),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::Sacrifice,
                players: PlayerSet::Opponent,
                filter: NONLAND,
                amount: 1,
                optional_skip: None,
            },
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::ReturnCraftedCreatureAttacking,
                players: PlayerSet::Controller,
                filter: NONLAND,
                amount: 1,
                optional_skip: Some(0),
            },
        ],
    }],
    ..CardRules::EMPTY
};
