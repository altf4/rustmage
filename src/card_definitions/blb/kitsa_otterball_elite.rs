const TAP: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[],
    sorcery: false,
    once_per_turn: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    keywords: Keywords::VIGILANCE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Noncreature,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Grow {
            subject: Subject::Source,
            power: 1,
            toughness: 1,
        }],
    }],
    activated: &[
        ActivatedAbility {
            program: &[
                Instruction::Draw(PlayerSet::Controller, 1),
                Instruction::Discard(PlayerSet::Controller, 1),
            ],
            ..TAP
        },
        ActivatedAbility {
            guard: ActivationGuard::SourcePowerAtLeast(3),
            cost: ActivationCost {
                generic: 2,
                mana_component: true,
                ..TAP.cost
            },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::YourSpellKinds {
                kinds: 4 | 8,
            })),
            program: &[Instruction::CopyTargetSpell {
                group: 0,
                retarget: true,
                exceptions: CopyExceptions::NONE,
            }],
            ..TAP
        },
    ],
    ..CardRules::EMPTY
};
