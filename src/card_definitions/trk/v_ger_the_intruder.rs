// Complete Oracle and pinned VGerTheIntruder.java reviewed. The hand look is
// private to the triggering controller; the other mode lasts to their next turn.
const OPPOSING_CREATURES: CreatureFilter = CreatureFilter {
    relation: 1,
    other: false,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 7),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersOrAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Look at target opponent's hand, then draw a card",
                targets: one(TargetRule::Player {
                    opponent_only: true,
                }),
                program: &[
                    Instruction::LookTargetHand,
                    Instruction::Draw(PlayerSet::Controller, 1),
                ],
            },
            SpellMode {
                text: "Creatures your opponents control get -1/-0 until your next turn",
                targets: TargetPlan::NONE,
                program: &[Instruction::CreatureStatsUntilNextTurn {
                    filter: OPPOSING_CREATURES,
                    power: -1,
                    toughness: 0,
                }],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
