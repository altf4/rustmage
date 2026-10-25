// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttackPlayer(CreatureFilter {
            relation: 0,
            other: false,
            subtype: Some("Merfolk"),
            keyword: Keywords::NONE,
            attacking: false,
            tokens: None,
            artifact: false,
        }),
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::StaticModifier(
            0,
            StaticModifier {
                own_turn: false,
                requires_life_gain: false,
                filter: CreatureFilter {
                    relation: 0,
                    other: true,
                    subtype: Some("Merfolk"),
                    keyword: Keywords::NONE,
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
                stats: Stats {
                    power: 1,
                    toughness: 1,
                },
            },
        )],
    }],
    extras: &CardExtras {
        static_modifiers: &[StaticModifier {
            own_turn: false,
            requires_life_gain: false,
            filter: CreatureFilter {
                relation: 0,
                other: true,
                subtype: Some("Merfolk"),
                keyword: Keywords::NONE,
                attacking: false,
                tokens: None,
                artifact: false,
            },
            stats: Stats {
                power: 1,
                toughness: 1,
            },
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
