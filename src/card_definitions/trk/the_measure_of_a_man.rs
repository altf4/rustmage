// Oracle and official Star Trek release notes reviewed; XMage class absent at the pinned revision.
const NONTOKEN_ARTIFACT_OR_CREATURE: TargetRule = TargetRule::Filtered {
    base: &TargetRule::Object {
        zone: TargetZone::Battlefield, kinds: 1 | 16, relation: -1,
        other: false, nonland: false, tapped: false,
        min_mana: 0, max_mana: u32::MAX,
    },
    predicate: &TargetPredicate::Nontoken,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::Single(TargetSpec {
                rule: NONTOKEN_ARTIFACT_OR_CREATURE, min: 0, max: 1,
            }),
            program: &[Instruction::ExileTargetsLinkedToSource],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens { token: Card(5296), amount: 1, tapped: false }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ReturnLinkedExileToBattlefield,
                Instruction::GainLife(PlayerSet::Controller, 2),
            ],
        },
    ],
    ..CardRules::EMPTY
};
