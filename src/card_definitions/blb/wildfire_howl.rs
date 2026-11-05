// One simultaneous damage event after the optional gift draw.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::NONE),
        &[
            Instruction::GiveGift,
            Instruction::DamageBattlefieldAndTargets {
                target_amount: 1,
                filters: &[BattlefieldFilter {
                    kinds: 1,
                    relation: -1,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: 4294967295,
                }],
                players: None,
                amount: 2,
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
    keywords: Keywords::NONE,
    extras: &CardExtras {
        gift: {
            const SPEC: GiftSpec = GiftSpec {
                kind: GiftKind::Card,
                token: None,
                promised_targets: TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
            };
            Some(SPEC)
        },
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
