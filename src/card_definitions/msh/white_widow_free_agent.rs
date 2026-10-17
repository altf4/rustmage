// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Put a +1/+1 counter on each of up to two target creatures.",
                targets: TargetPlan::Single(TargetSpec {
                    rule: TargetRule::Creature,
                    min: 0,
                    max: 2,
                }),
                program: &[Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                }],
            },
            SpellMode {
                text: "Return target artifact or enchantment card from your graveyard to your hand.",
                targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 48,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                })),
                program: &[Instruction::ReturnToHand],
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
    keywords: Keywords::NONE,
    ..CardRules::EMPTY
};
