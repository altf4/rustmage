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
    AttachedSubtype(&'static str),
    AttachmentStats(Stats),
    AttachmentKeywords(Keywords),
    AttachedTurnKeywords(Keywords),
    /// Stable index into the granting definition's immutable trigger array.
    AttachmentTrigger(u16),
    StaticModifier(StaticModifier),
    ScalingModifier(ScalingModifier),
    EnchantedModifier(EnchantedModifier),
    CounterKeyword(CounterKind, Keywords),
    DynamicStats(DynamicStats),
    BaseStatSetter(BaseStatSetter),
    LiveBase(LiveStaticEffect),
    LiveBonus(LiveStaticEffect),
    LiveKeywords(LiveStaticEffect),
    LiveBlockRestriction(LiveStaticEffect),
    Replacement(ReplacementSpec),
    CastTiming(CastFilter),
    CostReduction(SpellReduction),
    TargetCostReduction(TargetSpellReduction),
    CombatRestriction(CombatRestriction),
    EntryTriggerSuppression(u16),
    OpponentEntryTapped(u16),
}
