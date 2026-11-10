pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Puff Piece",
                targets: TargetPlan::Single(TargetSpec {
                    rule: TargetRule::Creature,
                    min: 0,
                    max: 2,
                }),
                program: &[Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                }],
            },
            SpellMode {
                text: "Investigative Journalism",
                targets: one(TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 1,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: 2,
                }),
                program: &[Instruction::ReturnToHand],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
