// Both triggering events use the same printed mana effect, a regular stack ability.
const MANA: &[Instruction] = &[Instruction::AddMana(ManaProduction::Tagged {
    production: &ManaProduction::any_color(2),
    quality: &ManaQuality {
        restriction: ManaRestriction::SpellManaValueAtLeast(4),
        spell_bonus: ManaSpellBonus::None,
        retention: ManaRetention::Step,
    },
})];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    extras: &CardExtras {
        transform: Some(TransformSpec {
            front: Card(243),
            back: Card(5348),
            front_name: "Ashling, Rekindled",
        }),
        ..CardExtras::EMPTY
    },
    deck_limit: None,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfTransforms,
            targets: TargetPlan::NONE,
            program: MANA,
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::Main1,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: MANA,
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::Main1,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPayMana {
                    cost: ManaCost {
                        colored: [0, 0, 0, 1, 0, 0],
                        generic: 0,
                    },
                    skip: 1,
                },
                Instruction::TransformSource,
            ],
        },
    ],
    ..CardRules::EMPTY
};
