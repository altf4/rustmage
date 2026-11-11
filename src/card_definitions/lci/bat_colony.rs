const BAT: Card = Card::from_catalogue_id(5276);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokensScaled {
                token: BAT,
                amount: AmountValue::CaveManaSpent,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SubtypeEntersYou("Cave"),
            targets: one(TargetRule::creature_you_control()),
            program: &[Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
    ],
    ..CardRules::EMPTY
};
