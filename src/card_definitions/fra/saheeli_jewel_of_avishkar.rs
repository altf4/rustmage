// Directly authored from pinned Oracle and XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Noncreature,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5266),
            amount: 1,
            tapped: false,
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledSubtypeKeywords {
            subtype: "Thopter",
            keywords: Keywords::HASTE,
        }],
    }],
    ..CardRules::EMPTY
};
