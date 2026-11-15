// Oracle and official Star Trek release notes reviewed; XMage class absent at the pinned revision.
const GREEN_OR_WHITE_CREATURE_CARD: TargetRule = TargetRule::Filtered {
    base: &TargetRule::Object {
        zone: TargetZone::Graveyard, kinds: 1, relation: 0,
        other: false, nonland: false, tapped: false,
        min_mana: 0, max_mana: u32::MAX,
    },
    predicate: &TargetPredicate::Colors(1 | 16),
};
const OUR_CREATURES: CreatureFilter = CreatureFilter {
    relation: 0, other: false, subtype: None, keyword: Keywords::NONE,
    attacking: false, tokens: None, artifact: false,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: one(GREEN_OR_WHITE_CREATURE_CARD),
            program: &[Instruction::ReturnToBattlefield],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: one(TargetRule::Creature),
            program: &[Instruction::DelayTargetEvent {
                event: DelayedTargetEvent::CombatDamageToPlayer,
                ability: 3,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::GainLifeScaled(PlayerSet::Controller, AmountValue::GreatestCreaturePower(OUR_CREATURES)),
                Instruction::CreateTokens { token: Card(5302), amount: 1, tapped: false },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::DelayedTargetEvent,
            targets: TargetPlan::NONE,
            program: &[Instruction::DrawScaled(PlayerSet::Controller, AmountValue::DamageEventAmount)],
        },
    ],
    ..CardRules::EMPTY
};
