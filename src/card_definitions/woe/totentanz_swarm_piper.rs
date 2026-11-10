const CREATE_RAT: &[Instruction] = &[Instruction::CreateTokens {
    token: Card(5323),
    amount: 1,
    tapped: false,
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfDies,
            targets: TargetPlan::NONE,
            program: CREATE_RAT,
        },
        TriggeredAbility {
            condition: TriggerCondition::CreatureDies {
                relation: CastRelation::You,
                other: true,
                nontoken: true,
                subtype: None,
            },
            targets: TargetPlan::NONE,
            program: CREATE_RAT,
        },
    ],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::Filtered {
            base: &TargetRule::Filtered {
                base: &TargetRule::creature_you_control(),
                predicate: &TargetPredicate::Subtype("Rat"),
            },
            predicate: &TargetPredicate::Attacking,
        }),
        program: &[Instruction::GrantKeywords {
            subject: Subject::Targets,
            keywords: Keywords::DEATHTOUCH,
        }],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
