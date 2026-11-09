const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::OwnTurnSourceAndKindCreatureWard {
        kinds: 32,
        keywords: Keywords::TRAMPLE.union(Keywords::LIFELINK),
        ward: 0,
    }],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 5),
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Granted(&TriggerCondition::Ward),
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterUnlessPay {
                cost: ManaCost {
                    colored: [0; 6],
                    generic: 2,
                },
                subject: CounterSubject::Related,
                exile: false,
                unpaid_skip: 0,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 32,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                min: 0,
                max: 1,
            }),
            program: &[Instruction::ReturnToBattlefieldWithCounter(
                CounterKind::Finality,
            )],
        },
    ],
    ..CardRules::EMPTY
};
