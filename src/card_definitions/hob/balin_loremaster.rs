// Oracle uses actual completed discards, rather than XMage's initial hand size.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Storied],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PermanentEntersYou {
            kinds: 0,
            nontoken: false,
            subtype: Some("Dwarf"),
            include_source: true,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::May { skip: 4 },
            Instruction::DiscardHandRememberCount(PlayerSet::Controller),
            Instruction::DrawMoved,
            Instruction::If {
                guard: StepGuard::Expression {
                    value: StatValue::HasEnduringStory,
                    minimum: Some(1),
                    maximum: None,
                },
                skip: 1,
            },
            Instruction::DamagePlayersScaled(PlayerSet::Opponent, AmountValue::MovedCount),
        ],
    }],
    ..CardRules::EMPTY
};
