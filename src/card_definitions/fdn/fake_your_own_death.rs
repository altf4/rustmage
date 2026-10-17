// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 2,
                toughness: 0,
            },
            Instruction::GrantUntilEndTurn { ability: 0 },
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Granted(&TriggerCondition::SelfDies),
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[
            Instruction::ReturnGrantedSource {
                counter: None,
                role: None,
            },
            Instruction::CreateTokens {
                token: Card(5302),
                amount: 1,
                tapped: false,
            },
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
