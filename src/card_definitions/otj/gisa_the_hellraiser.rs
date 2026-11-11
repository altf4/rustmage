const ZOMBIE_ROGUE: Card = Card::from_catalogue_id(5305);

const fn tribe(subtype: &'static str) -> CreatureFilter {
    CreatureFilter {
        relation: 0,
        other: false,
        subtype: Some(subtype),
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    }
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterRelatedUnlessPayManaLife {
                cost: generic(2),
                life: 2,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::OncePerTurn(&TriggerCondition::CommittedCrime),
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: ZOMBIE_ROGUE,
                amount: 2,
                tapped: true,
            }],
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::ControlledSubtypeKeywords {
                subtype: "Skeleton",
                keywords: Keywords::MENACE,
            },
            StaticAbilityFragment::ControlledSubtypeKeywords {
                subtype: "Zombie",
                keywords: Keywords::MENACE,
            },
            StaticAbilityFragment::CreatureGroupBonus {
                filters: &[tribe("Skeleton"), tribe("Zombie")],
                stats: Stats {
                    power: 1,
                    toughness: 1,
                },
            },
        ],
    }],
    ..CardRules::EMPTY
};
