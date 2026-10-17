// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Discard a card, then draw a card.",
                targets: TargetPlan::Single(TargetSpec::NONE),
                program: &[
                    Instruction::Discard(PlayerSet::Controller, 1),
                    Instruction::Draw(PlayerSet::Controller, 1),
                ],
            },
            SpellMode {
                text: "Create a Treasure token.",
                targets: TargetPlan::Single(TargetSpec::NONE),
                program: &[Instruction::CreateTokens {
                    token: Card(5302),
                    amount: 1,
                    tapped: false,
                }],
            },
            SpellMode {
                text: "Create a Wicked Role token attached to target creature you control.",
                targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                })),
                program: &[Instruction::CreateRole {
                    token: Card(5248),
                    subject: Subject::Targets,
                }],
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
    keywords: Keywords::HASTE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::HASTE)],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::HASTE)],
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
