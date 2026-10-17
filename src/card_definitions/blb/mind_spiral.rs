// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: false,
        })),
        &[
            Instruction::GiveGift,
            Instruction::Draw(PlayerSet::Targeted, 3),
            Instruction::IfGift {
                promised: true,
                skip: 3,
            },
            Instruction::SelectGroup(1),
            Instruction::Tap,
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::Stun,
                amount: 1,
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
                kind: GiftKind::TappedFish,
                token: Some(Card(5317)),
                promised_targets: TargetPlan::Groups(&[
                    TargetSpec::one(TargetRule::Player {
                        opponent_only: false,
                    }),
                    TargetSpec::one(TargetRule::Object {
                        zone: TargetZone::Battlefield,
                        kinds: 1,
                        relation: 1,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    }),
                ]),
            };
            Some(SPEC)
        },
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
