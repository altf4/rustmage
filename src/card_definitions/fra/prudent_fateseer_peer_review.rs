// Directly authored from complete pinned Oracle and XMage; gated pending validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 4),

    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OncePerTurn(&TriggerCondition::ScryOrSurveil),
        targets: TargetPlan::NONE,
        program: &[Instruction::GrowCreatures {
            filter: CreatureFilter {
                relation: 0,
                other: false,
                subtype: None,
                keyword: Keywords::NONE,
                attacking: false,
                tokens: None,
                artifact: false,
            },
            power: 1,
            toughness: 0,
        }],
    }],
    extras: &CardExtras {
        enters_prepared: true,
        prepared_spell: Some(Card(5446)),
        cost_alternatives: &[
            ManaCost {
                colored: [1, 1, 0, 0, 0, 0],
                generic: 1,
            },
            ManaCost {
                colored: [0, 2, 0, 0, 0, 0],
                generic: 1,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
