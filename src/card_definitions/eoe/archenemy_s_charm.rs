// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Exile target creature or planeswalker.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 65,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: 4294967295,
            })),
            program: &[Instruction::Exile],
        },
        SpellMode {
            text: "Return one or two target creature and/or planeswalker cards from your graveyard to your hand.",
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 65,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                min: 1,
                max: 2,
            }),
            program: &[Instruction::ReturnToHand],
        },
        SpellMode {
            text: "Put two +1/+1 counters on target creature you control. It gains lifelink until end of turn.",
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
            program: &[
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: 2,
                },
                Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords(64),
                },
            ],
        },
    ]),
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
    ..CardRules::EMPTY
};
