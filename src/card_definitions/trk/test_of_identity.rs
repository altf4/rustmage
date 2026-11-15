// Complete Oracle and pinned TestOfIdentity.java reviewed. The second mode
// discards if possible and draws only after an actual discard event.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "I Am Klingon — target Klingon gets +1/+1 and double strike",
            targets: one(TargetRule::Filtered {
                base: &TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    relation: -1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                predicate: &TargetPredicate::Subtype("Klingon"),
            }),
            program: &[
                Instruction::Grow { subject: Subject::Targets, power: 1, toughness: 1 },
                Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords::DOUBLE_STRIKE,
                },
            ],
        },
        SpellMode {
            text: "I Am Starfleet — discard a card; if you do, draw two",
            targets: TargetPlan::NONE,
            program: &[
                Instruction::DiscardRememberCount { players: PlayerSet::Controller, amount: 1 },
                Instruction::IfAmountAtLeast { value: AmountValue::MovedCount, minimum: 1, skip: 1 },
                Instruction::Draw(PlayerSet::Controller, 2),
            ],
        },
    ]),
    ..CardRules::EMPTY
};
