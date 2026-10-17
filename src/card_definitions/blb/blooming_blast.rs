// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::GiveGift,
            Instruction::IfGift {
                promised: false,
                skip: 1,
            },
            Instruction::Damage(2),
            Instruction::IfGift {
                promised: true,
                skip: 1,
            },
            Instruction::DamageWithController {
                damage: 2,
                controller_damage: 3,
            },
        ],
    ),
    triggers: &[],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords(0),
    extras: &CardExtras {
        gift: {
            const SPEC: GiftSpec = GiftSpec {
                kind: GiftKind::Treasure,
                token: Some(Card(5302)),
                promised_targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            };
            Some(SPEC)
        },
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
