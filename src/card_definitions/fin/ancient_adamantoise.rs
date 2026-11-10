const REDIRECT: ReplacementSpec = ReplacementSpec::RedirectControlledDamageToSource;

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(8, 20),
    keywords: Keywords::VIGILANCE,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterUnlessPayCosts {
                subject: CounterSubject::Related,
                costs: &[ResolutionCost {
                    mana: generic(3),
                    ..ResolutionCost::FREE
                }],
                exile: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDies,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ExileDepartedSource,
                Instruction::CreateTokens {
                    token: Card(5302),
                    amount: 10,
                    tapped: true,
                },
            ],
        },
    ],
    static_groups: &[
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::RetainDamageDuringCleanup],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::Replacement(0, REDIRECT)],
        },
    ],
    extras: &CardExtras {
        replacement_effects: &[REDIRECT],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
