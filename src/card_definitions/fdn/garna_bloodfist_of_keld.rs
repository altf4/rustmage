// Direct typed behavior from complete Oracle and pinned XMage sources.
const OTHER_YOURS: CreatureFilter = CreatureFilter {
    relation: 0,
    other: true,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    // Disjoint predicates select the printed ability's branch at the event.
    // Only one packet exists per death; no extra death facts live in game state.
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::CreatureDiesAttackState {
                filter: OTHER_YOURS,
                attacking: true,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::CreatureDiesAttackState {
                filter: OTHER_YOURS,
                attacking: false,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::DamagePlayers(PlayerSet::Opponent, 1)],
        },
    ],
    ..CardRules::EMPTY
};
