// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Search(SearchSpec {
            filter: SearchFilter::AnyOf(&[
                SearchFilter::Subtype("Lesson"),
                SearchFilter::Subtype("Noble"),
            ]),
            maximum: 1,
            destination: SearchDestination::Hand,
            tapped: false,
            reveal: true,
        })],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::ControlledKindAndSubtypeKeywords {
                kind: 1,
                subtype: "Noble",
                keywords: Keywords::LIFELINK,
            },
            StaticAbilityFragment::ControlledInstantSorcerySubtypeKeywords {
                subtype: "Lesson",
                keywords: Keywords::LIFELINK,
            },
        ],
    }],
    ..CardRules::EMPTY
};
