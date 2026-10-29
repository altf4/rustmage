// Direct Oracle and pinned XMage behavior; gated pending interaction validation.
const DAMAGE_TARGET: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1 | 64,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
}));
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(7, 5),
    keywords: Keywords::TRAMPLE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::TRAMPLE)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        // Each entry is exactly one of the three possible pairs, in printed order.
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Deal 4 damage to target creature or planeswalker; create a Cadet.",
                targets: DAMAGE_TARGET,
                program: &[
                    Instruction::Damage(4),
                    Instruction::CreateTokens {
                        token: Card(5298),
                        amount: 1,
                        tapped: false,
                    },
                ],
            },
            SpellMode {
                text: "Deal 4 damage to target creature or planeswalker; draw a card.",
                targets: DAMAGE_TARGET,
                program: &[
                    Instruction::Damage(4),
                    Instruction::Draw(PlayerSet::Controller, 1),
                ],
            },
            SpellMode {
                text: "Create a Cadet; draw a card.",
                targets: TargetPlan::NONE,
                program: &[
                    Instruction::CreateTokens {
                        token: Card(5298),
                        amount: 1,
                        tapped: false,
                    },
                    Instruction::Draw(PlayerSet::Controller, 1),
                ],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
