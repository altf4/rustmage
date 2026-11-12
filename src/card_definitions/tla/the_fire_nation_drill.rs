// Direct typed behavior from the pinned Oracle and XMage sources.
const HEXPROOF_AND_INDESTRUCTIBLE: Keywords = Keywords::INDESTRUCTIBLE
    .union(Keywords::HEXPROOF)
    .union(Keywords::HEXPROOF_MONOCOLORED)
    .union(Keywords::HEXPROOF_OWN_COLORS)
    .union(Keywords::hexproof_from_color(ChoiceColor::White))
    .union(Keywords::hexproof_from_color(ChoiceColor::Blue))
    .union(Keywords::hexproof_from_color(ChoiceColor::Black))
    .union(Keywords::hexproof_from_color(ChoiceColor::Red))
    .union(Keywords::hexproof_from_color(ChoiceColor::Green));

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: Keywords::TRAMPLE,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::MayTapSourceReflexive { ability: 1 }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: one(TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::PowerAtMost(4),
            }),
            program: &[Instruction::Destroy],
        },
    ],
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                generic: 1,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::RemovePermanentGroupKeywords {
                filters: &[BattlefieldFilter {
                    kinds: 0,
                    relation: 1,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                }],
                keywords: HEXPROOF_AND_INDESTRUCTIBLE,
            }],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost::ZERO,
            targets: TargetPlan::NONE,
            program: &[Instruction::Crew { minimum: 2 }],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 6,
            toughness: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
