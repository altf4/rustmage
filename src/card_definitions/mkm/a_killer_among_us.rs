use rustmage_core::engine::replacements::TokenGroupView;

const ATTACKING_TOKEN: TargetRule = TargetRule::Filtered {
    base: &TargetRule::Filtered {
        base: &TargetRule::Creature,
        predicate: &TargetPredicate::Token,
    },
    predicate: &TargetPredicate::Attacking,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::CreateTokenBatch(&[
                TokenGroupView {
                    token: Card(5254),
                    amount: 1,
                    tapped: false,
                },
                TokenGroupView {
                    token: Card(5398),
                    amount: 1,
                    tapped: false,
                },
                TokenGroupView {
                    token: Card(5324),
                    amount: 1,
                    tapped: false,
                },
            ]),
            Instruction::ChooseSecretCreatureType(&["Human", "Merfolk", "Goblin"]),
        ],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::RevealChosenSecretCreatureType,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(ATTACKING_TOKEN)),
        program: &[
            Instruction::IfTargetHasRevealedCreatureType { skip: 2 },
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 3,
            },
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::DEATHTOUCH,
            },
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
