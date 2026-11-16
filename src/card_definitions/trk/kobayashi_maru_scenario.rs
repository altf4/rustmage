// Pinned XMage: each upkeep chooses a mode that this exact permanent has not chosen.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Upkeep,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ModalOnce(&[
            SpellMode {
                text: "Hail — Return up to one target creature an opponent controls with mana value 3 or less to its owner's hand.",
                targets: TargetPlan::Single(TargetSpec {
                    rule: TargetRule::Object {
                        zone: TargetZone::Battlefield,
                        kinds: 1,
                        relation: 1,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: 3,
                    },
                    min: 0,
                    max: 1,
                }),
                program: &[Instruction::ReturnToHand],
            },
            SpellMode {
                text: "Sensors — Draw two cards.",
                targets: TargetPlan::NONE,
                program: &[Instruction::Draw(PlayerSet::Controller, 2)],
            },
            SpellMode {
                text: "Transport — Create two 1/1 red Officer creature tokens.",
                targets: TargetPlan::NONE,
                program: &[Instruction::CreateTokens {
                    token: Card(6008),
                    amount: 2,
                    tapped: false,
                }],
            },
            SpellMode {
                text: "Abandon Ship — You lose the game.",
                targets: TargetPlan::NONE,
                program: &[Instruction::LoseGame(PlayerSet::Controller)],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
