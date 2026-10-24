// Complete typed modes: zero, four singles and six unordered pairs.
const ARTIFACT: TargetSpec = TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 16,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
});
const CREATURE: TargetSpec = TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 1,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
});
const ENCHANTMENT: TargetSpec = TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 32,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
});
const LAND: TargetSpec = TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 2,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
});
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Choose no modes",
            targets: TargetPlan::NONE,
            program: &[],
        },
        SpellMode {
            text: "Artifact",
            targets: TargetPlan::Single(ARTIFACT),
            program: &[Instruction::ReturnToHand],
        },
        SpellMode {
            text: "Creature",
            targets: TargetPlan::Single(CREATURE),
            program: &[Instruction::ReturnToHand],
        },
        SpellMode {
            text: "Enchantment",
            targets: TargetPlan::Single(ENCHANTMENT),
            program: &[Instruction::ReturnToHand],
        },
        SpellMode {
            text: "Land",
            targets: TargetPlan::Single(LAND),
            program: &[Instruction::ReturnToHand],
        },
        SpellMode {
            text: "Artifact and creature",
            targets: TargetPlan::Groups(&[ARTIFACT, CREATURE]),
            program: &[Instruction::ReturnAllTargetsToHand],
        },
        SpellMode {
            text: "Artifact and enchantment",
            targets: TargetPlan::Groups(&[ARTIFACT, ENCHANTMENT]),
            program: &[Instruction::ReturnAllTargetsToHand],
        },
        SpellMode {
            text: "Artifact and land",
            targets: TargetPlan::Groups(&[ARTIFACT, LAND]),
            program: &[Instruction::ReturnAllTargetsToHand],
        },
        SpellMode {
            text: "Creature and enchantment",
            targets: TargetPlan::Groups(&[CREATURE, ENCHANTMENT]),
            program: &[Instruction::ReturnAllTargetsToHand],
        },
        SpellMode {
            text: "Creature and land",
            targets: TargetPlan::Groups(&[CREATURE, LAND]),
            program: &[Instruction::ReturnAllTargetsToHand],
        },
        SpellMode {
            text: "Enchantment and land",
            targets: TargetPlan::Groups(&[ENCHANTMENT, LAND]),
            program: &[Instruction::ReturnAllTargetsToHand],
        },
    ]),
    ..CardRules::EMPTY
};
