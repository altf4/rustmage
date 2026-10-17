// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Tap target creature.",
                targets: TargetPlan::Single(TargetSpec {
                    rule: TargetRule::Creature,
                    min: 1,
                    max: 1,
                }),
                program: &[Instruction::Tap],
            },
            SpellMode {
                text: "Untap target creature.",
                targets: TargetPlan::Single(TargetSpec {
                    rule: TargetRule::Creature,
                    min: 1,
                    max: 1,
                }),
                program: &[Instruction::Untap],
            },
            SpellMode {
                text: "Draw a card, then discard a card.",
                targets: TargetPlan::Single(TargetSpec::NONE),
                program: &[
                    Instruction::Draw(PlayerSet::Controller, 1),
                    Instruction::Discard(PlayerSet::Controller, 1),
                ],
            },
        ])],
    }],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords(32),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(32))],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(32))],
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
