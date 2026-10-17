// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 0,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            predicate: &TargetPredicate::Permanent,
        })),
        &[Instruction::GiveGift, Instruction::ReturnToHand],
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
                kind: GiftKind::Card,
                token: None,
                promised_targets: TargetPlan::Single(TargetSpec {
                    rule: TargetRule::Filtered {
                        base: &TargetRule::Object {
                            zone: TargetZone::Graveyard,
                            kinds: 0,
                            relation: 0,
                            other: false,
                            nonland: false,
                            tapped: false,
                            min_mana: 0,
                            max_mana: u32::MAX,
                        },
                        predicate: &TargetPredicate::Permanent,
                    },
                    min: 2,
                    max: 2,
                }),
            };
            Some(SPEC)
        },
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
