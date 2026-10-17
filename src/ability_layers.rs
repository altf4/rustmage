//! Immutable static-effect provenance; shared fragments retain one source identity.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub(crate) enum AbilityLayer {
    Type = 4,
    Color = 5,
    Ability = 6,
    Cda = 70,
    Base = 71,
    Modify = 72,
    Rule = 100,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct StaticAbilityGroup {
    /// Original printed line, not the index of a synthetic residual fragment.
    pub line: u16,
    pub first_layer: AbilityLayer,
    pub fragments: &'static [StaticAbilityFragment],
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum StaticAbilityFragment {
    IntrinsicKeywords(Keywords),
    Changeling,
    /// Predefined Cursed Role sets both axes in layer 7b.
    PredefinedRoleBase,
    AttachedSubtype(&'static str),
    AttachmentStats(Stats),
    AttachmentKeywords(Keywords),
    AttachedTurnKeywords(Keywords),
    /// Stable index into the granting definition's immutable trigger array.
    AttachmentTrigger(u16),
    KeywordGrant(u16, KeywordGrant),
    StaticModifier(u16, StaticModifier),
    ScalingModifier(u16, ScalingModifier),
    EnchantedModifier(u16, EnchantedModifier),
    CounterKeyword(u16, CounterKind, Keywords),
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
    EntryTriggerSuppression(u16, u16),
    OpponentEntryTapped(u16, u16),
}
