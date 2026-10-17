// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfOffspring,
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::CreateTokens {
                token: Card(5308),
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::OncePerTurn(&TriggerCondition::SelfTargeted {
                relation: CastRelation::You,
                spells_only: false,
            }),
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::GrowCreatures {
                filter: CreatureFilter {
                    relation: 0,
                    other: false,
                    subtype: Some("Mouse"),
                    keyword: Keywords(0),
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
                power: 1,
                toughness: 0,
            }],
        },
    ],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords(0),
    extras: &CardExtras {
        offspring_cost: Some(ManaCost {
            colored: [0, 0, 0, 0, 0, 0],
            generic: 2,
        }),
        offspring_copy: Some(Card(5308)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
