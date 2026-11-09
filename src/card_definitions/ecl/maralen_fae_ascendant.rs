// Direct typed behavior from full Oracle, pinned XMage watcher and current casting rules.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 5),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FilteredEntersYou(EntryPredicate::SourceOrSubtypeAny(&[
            "Elf", "Faerie",
        ])),
        targets: one(TargetRule::Player {
            opponent_only: true,
        }),
        program: &[Instruction::ExileTargetLibraryLinked(2)],
    }],
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::LinkedExileFreeCastBySubtypeUnion(&[
                "Elf", "Faerie",
            ])],
        },
    ],
    extras: &CardExtras {
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
            },
            StaticAbilityGroup {
                line: 2,
                first_layer: AbilityLayer::Rule,
                fragments: &[StaticAbilityFragment::LinkedExileFreeCastBySubtypeUnion(&[
                    "Elf", "Faerie",
                ])],
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
