use crate::engine::SelfGrantedAbilityKind;

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5327),
                amount: 2,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: &[Instruction::GrantPermanentSelfAbility(
                SelfGrantedAbilityKind::ControlledPermanentTapMana {
                    kinds: 1,
                    colors: 1 | 8 | 16,
                },
            )],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: &[Instruction::Search(SearchSpec {
                filter: SearchFilter::Subtype("Dinosaur"),
                maximum: 1,
                destination: SearchDestination::Hand,
                tapped: false,
                reveal: true,
            })],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 4 },
            targets: TargetPlan::NONE,
            program: &[Instruction::GrantPermanentGroupKeywords {
                filters: &[BattlefieldFilter {
                    kinds: 0,
                    relation: 0,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: Some("Dinosaur"),
                    max_mana: u32::MAX,
                }],
                keywords: Keywords::DOUBLE_STRIKE.union(Keywords::TRAMPLE),
            }],
        },
    ],
    ..CardRules::EMPTY
};
