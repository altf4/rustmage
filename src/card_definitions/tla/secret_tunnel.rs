// Direct typed behavior from the pinned Oracle and XMage sources.
const TAP: ActivatedAbility = ActivatedAbility {
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    exhaust: false,
    sorcery: false,
    once_per_turn: false,
    targets: TargetPlan::NONE,
    program: &[],
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    keywords: Keywords::CANT_BE_BLOCKED,
    activated: &[
        ActivatedAbility {
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 0, 0, 1,
            ]))],
            ..TAP
        },
        ActivatedAbility {
            cost: ActivationCost {
                generic: 4,
                mana_component: true,
                ..TAP.cost
            },
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Grouped {
                    base: &TargetRule::creature_you_control(),
                    constraint: TargetGroupConstraint::SharedCreatureType,
                },
                min: 2,
                max: 2,
            }),
            program: &[Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::CANT_BE_BLOCKED,
            }],
            ..TAP
        },
    ],
    ..CardRules::EMPTY
};
