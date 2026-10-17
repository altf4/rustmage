//! Immutable static-effect provenance; shared fragments retain one source identity.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub(crate) enum AbilityLayer {
    Type = 4,
    #[expect(
        dead_code,
        reason = "Layer-five source identities are retained for typed legacy definitions"
    )]
    Color = 5,
    Ability = 6,
    Cda = 70,
    Base = 71,
    Modify = 72,
    Rule = 100,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct StaticAbilityGroup {
    /// Stable source-local ability identity; legacy projections use printed lines.
    #[expect(
        dead_code,
        reason = "Immutable origin identity is retained for typed definition migration and diagnostics"
    )]
    pub line: u16,
    pub first_layer: AbilityLayer,
    pub fragments: &'static [StaticAbilityFragment],
}

#[derive(Clone, Copy, Debug)]
#[expect(
    dead_code,
    reason = "Complete immutable typed origin payloads are preserved while legacy fast tables supply some values; do not discard during direct-definition migration"
)]
pub(crate) enum StaticAbilityFragment {
    PlayerHexproof,
    ControlledSpellsUncounterable(u16),
    LifeGainAdditional(u16),
    CantGainLife,
    NoMaximumHandSize,
    AdditionalLandPlays(u16),
    IntrinsicKeywords(Keywords),
    Changeling,
    /// Predefined Cursed Role sets both axes in layer 7b.
    PredefinedRoleBase,
    RemoveAttachedAbilities,
    AttachedUntap(UntapRestriction),
    SelfUntap(UntapRestriction),
    AttachedSubtype(&'static str),
    AttachmentStats(Stats),
    AttachmentTurnStats(Stats),
    AttachmentKeywords(Keywords),
    AttachmentRule(Keywords),
    AttachedTurnKeywords(Keywords),
    /// Stable index into the granting definition's immutable trigger array.
    AttachmentTrigger(u16),
    /// Grants an independent tap mana ability to the attached permanent.
    AttachmentMana(ManaProduction),
    KeywordGrant(u16, KeywordGrant),
    StaticRuleGrant(u16, KeywordGrant),
    StaticModifier(u16, StaticModifier),
    ScalingModifier(u16, ScalingModifier),
    EnchantedModifier(u16, EnchantedModifier),
    CounterKeyword(u16, CounterKind, Keywords),
    /// One union effect: Skeletons and other Zombies, never twice for changeling.
    DeathBaronLord,
    CounterKeywordOwnTurn(CounterKind, Keywords),
    DynamicStats(u16, DynamicStats),
    BaseStatSetter(u16, BaseStatSetter),
    LiveBase(u16, LiveStaticEffect),
    LiveBonus(u16, LiveStaticEffect),
    LiveKeywords(u16, LiveStaticEffect),
    LiveBlockRestriction(u16, LiveStaticEffect),
    Replacement(u16, ReplacementSpec),
    CastTiming(u16, CastFilter),
    CostReduction(u16, SpellReduction),
    TargetCostReduction(u16, TargetSpellReduction),
    CombatRestriction(u16, CombatRestriction),
    /// Source attacks every combat when independently able to attack.
    AttacksEachCombat,
    /// The source's evasion ability excludes blockers with an effective subtype.
    CannotBeBlockedBySubtype(&'static str),
    EntryTriggerSuppression(u16, u16),
    OpponentEntryTapped(u16, u16),
    ControlledEntryUntapped,
    StaticCastRestriction(u16, u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UntapRestriction {
    All,
    ControllerStep,
}
