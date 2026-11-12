pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::VIGILANCE,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                Keywords::VIGILANCE,
            )],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::ControlledSubtypeTapMana {
                subtype: "Frog",
                production: ManaProduction::Tagged {
                    production: &ManaProduction::Choice {
                        mask: 2 | 16,
                        amount: 1,
                    },
                    quality: &ManaQuality {
                        restriction: ManaRestriction::CreatureSpells,
                        spell_bonus: ManaSpellBonus::None,
                        retention: ManaRetention::Step,
                    },
                },
            }],
        },
    ],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureEnters {
            relation: CastRelation::You,
            other: false,
        },
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Filtered {
                base: &TargetRule::creature_you_control(),
                predicate: &TargetPredicate::ManaValueLessThanEvent { other: false },
            },
            min: 0,
            max: 1,
        }),
        program: &[Instruction::ReturnToHand],
    }],
    ..CardRules::EMPTY
};
