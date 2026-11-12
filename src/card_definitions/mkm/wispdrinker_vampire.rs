pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OtherFilteredEntersYou(EntryPredicate::CreaturePowerAtMost(2)),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::LoseLife(PlayerSet::Opponent, 1),
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            colored: [1, 0, 1, 0, 0, 0],
            generic: 5,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        sorcery: false,
        once_per_turn: false,
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantCreaturesWithPowerAtMost {
            filter: CreatureFilter {
                relation: 0,
                other: false,
                subtype: None,
                keyword: Keywords::NONE,
                attacking: false,
                tokens: None,
                artifact: false,
            },
            maximum: 2,
            keywords: Keywords::DEATHTOUCH.union(Keywords::LIFELINK),
        }],
    }],
    ..CardRules::EMPTY
};
