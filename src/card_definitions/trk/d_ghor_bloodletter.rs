// Pinned Oracle and complete XMage class: modal entry affects all other creatures or sacrifices.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "All other creatures get -1/-1 until end of turn.",
                targets: TargetPlan::NONE,
                program: &[Instruction::GrowCreatures {
                    filter: CreatureFilter {
                        relation: -1,
                        other: true,
                        subtype: None,
                        keyword: Keywords::NONE,
                        attacking: false,
                        tokens: None,
                        artifact: false,
                    },
                    power: -1,
                    toughness: -1,
                }],
            },
            SpellMode {
                text: "Each player sacrifices a creature of their choice.",
                targets: TargetPlan::NONE,
                program: &[Instruction::ChooseObjects {
                    kind: EffectChoiceKind::Sacrifice,
                    players: PlayerSet::Each,
                    filter: SacrificeFilter {
                        keywords: Keywords::NONE,
                        source_only: false,
                        kinds: 1,
                        nonland: false,
                        other: false,
                        tokens: None,
                        artifact_enchantment_or_token: false,
                        subtype: None,
                    },
                    amount: 1,
                    optional_skip: None,
                }],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
