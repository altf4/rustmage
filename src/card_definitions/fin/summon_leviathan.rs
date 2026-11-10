// Direct typed behavior from the paired Oracle and pinned XMage sources.
const SEA_CREATURES: &[&str] = &["Kraken", "Leviathan", "Merfolk", "Octopus", "Serpent"];
const ATTACKS_THIS_TURN: &[Instruction] =
    &[Instruction::ScheduleAttackTriggerThisTurn { ability: 3 }];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: &[Instruction::ReturnCreaturesWithoutSubtypes(SEA_CREATURES)],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: ATTACKS_THIS_TURN,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: ATTACKS_THIS_TURN,
        },
        TriggeredAbility {
            condition: TriggerCondition::DelayedCreatureAttacksSubtypeAny(SEA_CREATURES),
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterUnlessPayCosts {
                subject: CounterSubject::Related,
                costs: &[ResolutionCost {
                    mana: ManaCost {
                        generic: 2,
                        ..ManaCost::ZERO
                    },
                    ..ResolutionCost::FREE
                }],
                exile: false,
            }],
        },
    ],
    ..CardRules::EMPTY
};
