// Every discarded card independently announces one mode unused by this object this turn.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Discarded {
            relation: CastRelation::You,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ModalOncePerTurn(&[
            SpellMode {
                text: "Draw a card.",
                targets: TargetPlan::NONE,
                program: &[Instruction::Draw(PlayerSet::Controller, 1)],
            },
            SpellMode {
                text: "Create a Treasure token.",
                targets: TargetPlan::NONE,
                program: &[Instruction::CreateTokens {
                    token: Card(5302),
                    amount: 1,
                    tapped: false,
                }],
            },
            SpellMode {
                text: "Each opponent loses 3 life.",
                targets: TargetPlan::NONE,
                program: &[Instruction::LoseLife(PlayerSet::Opponent, 3)],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
