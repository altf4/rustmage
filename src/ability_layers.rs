//! Immutable static-effect provenance; shared fragments retain one source identity.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub(crate) enum AbilityLayer {
    Control = 2,
    Text = 3,
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

/// Qualifications belong to the ability's actual permanent source, not the
/// immutable card which supplied a granted program.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AdditionalTriggerScope {
    ControlledSubtype(&'static str),
    EquippedCreature,
}

#[derive(Clone, Copy, Debug)]
#[expect(
    dead_code,
    reason = "Complete immutable typed origin payloads are preserved while legacy fast tables supply some values; do not discard during direct-definition migration"
)]
pub(crate) enum StaticAbilityFragment {
    /// Cast the exact card exiled by this source's linked hand-reveal ability at threshold.
    CastLinkedExileAtThreshold,
    /// Controlled instant and sorcery spells have split second.
    ControlledInstantSorcerySplitSecond,
    /// Permanents controlled by this source's controller cannot be sacrificed.
    ControlledPermanentsCannotBeSacrificed,
    /// Cards in graveyards cannot be exiled by any player.
    GraveyardCardsCannotBeExiled,
    /// Replace a controlled permanent's move to a graveyard with no move.
    PreventControlledPermanentGraveyardMove,
    /// Replace this controller's life payments with top-library exile when possible.
    ReplaceLifePaymentWithLibraryExile,
    /// During combat no spells or nonmana activated abilities may be started.
    NoSpellsOrNonmanaDuringCombat,

    /// Grants its controller the permanent enduring-story designation at threshold.
    Storied,
    /// This source continuously acquires activations from matching cards in its controller graveyard.
    BorrowGraveyardActivations(&'static str),
    /// This creature can attack despite defender after its controller scries or surveils.
    AttackWithoutDefenderIfScried,
    /// Controlled creatures may attack despite defender.
    ControlledAttackWithoutDefender,
    /// This source cannot be attacked while attached to a creature.
    AttachedCreaturePreventsSourceAttack,
    /// Controlled creatures with greater toughness assign combat damage using toughness.
    ControlledCombatDamageByGreaterToughness,
    /// This source assigns combat damage using the magnitude of negative power.
    SourceNegativePowerCombatDamage,
    /// Replace each nonexempt controller draw with two draws.
    DoubleDrawExceptFirst,
    /// Increase opponents' noncreature spell costs by generic mana.
    OpponentNoncreatureSpellTax(u32),
    /// Generic cost per creature declared attacking this source's controller.
    AttackControllerTax {
        amount: u32,
        condition: StatValue,
    },
    AdditionalTriggers {
        scope: AdditionalTriggerScope,
        condition: StatValue,
    },
    FreeHandCast,
    FreeHandCastAtMostControlledCreatures,
    GrantInstantSorceryStorm {
        ability: u16,
    },
    OwnTurnFreeHandInstantSorcery,
    HandMiracle {
        filter: CastFilter,
        ability: u16,
    },
    CascadeFromHand {
        ability: u16,
    },
    LookAtLibraryTop,
    CastCreatureFromLibraryTop,
    PlayLandFromLibraryTop,
    PlayLandFromGraveyard,
    SpendAnyManaForCreatures,
    SpellImprovise(CastFilter),
    GraveyardPermanentTypes,
    ForeignStashPlay,
    ControlAttached,
    AttachedCharacteristics(&'static AttachedCharacteristics),
    ChosenCreatureType,
    /// This land is the chosen basic land type (CR305.7).
    ChosenBasicLandType,
    OtherSubtypeEntryCounters(&'static str),
    ChosenModifier(ChosenModifier),
    /// Grant keywords to creatures of the source's chosen creature type.
    ChosenTypeKeywords(Keywords),
    /// Grant keywords to controlled battlefield permanents matching a live kind mask.
    ControlledPermanentKeywords {
        kinds: u16,
        keywords: Keywords,
    },
    /// Grant the immutable activated template to controlled planeswalkers.
    ControlledPlaneswalkerActivation(u16),
    /// Planeswalkers this permanent's controller controls survive zero loyalty.
    ControlledPlaneswalkersSurviveZeroLoyalty,
    /// Grant controlled planeswalkers the one-attacker restriction.
    ControlledPlaneswalkerAttackLimit,
    ControlledSubtypeKeywords {
        subtype: &'static str,
        keywords: Keywords,
    },
    PlayerHexproof,
    ActivateControlledCreaturesAsHaste,
    PlayerHexproofIfSourceCounter {
        kind: CounterKind,
        minimum: u64,
    },
    /// Real immutable trigger template, present only on the harnessed source.
    HarnessedTrigger(u16),
    CantLoseGameAndOpponentsCantWin,
    QualifiedHexproof(SourceCharacteristicFilter),
    ControlledSpellsUncounterable(u16),
    ControlledSpellsCasualty {
        kinds: u16,
        minimum: u16,
    },
    LifeGainAdditional(u16),
    CantGainLife,
    DamageCantBePrevented,
    NoMaximumHandSize,
    MaximumHandSize(u16),
    AdditionalLandPlays(u16),
    /// A live earlier-layer condition; allowance consumption stays turn-wide.
    ConditionalAdditionalLandPlays {
        condition: StatValue,
        amount: u16,
    },
    IntrinsicKeywords(Keywords),
    Changeling,
    /// Predefined Cursed Role sets both axes in layer 7b.
    PredefinedRoleBase,
    RemoveAttachedAbilities,
    AttachedUntap(UntapRestriction),
    /// Prevent counters on the enchanted/equipped permanent.
    AttachedCounterProhibition,
    /// Prevent counters on all permanents matching these card-type bits.
    CounterProhibition(u16),
    /// Players matching this relation cannot receive the specified counter kind.
    PlayerCounterProhibition {
        kind: crate::cards::PlayerCounterKind,
        relation: crate::cards::CastRelation,
    },
    SelfUntap(UntapRestriction),
    /// Uses exact-object +1/+1 placement history for the source's current controller.
    SourcePlusOnePlacedKeywords(Keywords),
    /// Source-only ability grant whose earlier-layer condition remains live.
    SourceConditionalKeywords {
        condition: StatValue,
        keywords: Keywords,
    },
    /// Current matching recipients with exact +1/+1 placement history by this controller.
    PlusOnePlacedKeywords {
        filter: CountFilter,
        keywords: Keywords,
    },
    SourceCounterKeywords {
        kind: CounterKind,
        minimum: u64,
        keywords: Keywords,
    },
    SourceCounterSubtype {
        kind: CounterKind,
        minimum: u64,
        subtype: &'static str,
        keywords: Keywords,
    },
    /// This source becomes a creature with these subtypes and keywords while it
    /// has the required counters, optionally only on its controller's turn. Its type change
    /// starts in layer four; the same ability's base stats are a separate
    /// BaseStatSetter fragment in its group.
    SourceCounterCreature {
        own_turn: bool,
        kind: CounterKind,
        minimum: u64,
        subtypes: &'static [&'static str],
        keywords: Keywords,
    },
    /// One source-local conditional effect may span type and stat layers.
    SourceGraveyardAllTypes {
        kind: u16,
        minimum: u16,
    },
    SourceGraveyardBonus {
        kind: u16,
        minimum: u16,
        stats: Stats,
    },
    AttachedSubtype(&'static str),
    AttachedLegendary,
    AttachmentStats(Stats),
    AttachmentTurnStats(Stats),
    AttachmentKeywords(Keywords),
    AttachmentRule(Keywords),
    AttachedTurnKeywords(Keywords),
    /// Stable index into the granting definition's immutable trigger array.
    AttachmentTrigger(u16),
    /// A real Ward instance granted by a live conditional static ability.
    ConditionalWard {
        condition: StatValue,
        filter: CountFilter,
        ability: u16,
    },
    AttachmentActivation(u16),
    /// Grants an independent tap mana ability to the attached permanent.
    AttachmentMana(ManaProduction),
    AttachmentSacrificeMana(ManaProduction),
    LandMana {
        recipients: LandManaRecipients,
        production: ManaProduction,
    },
    KeywordGrant(u16, KeywordGrant),
    StaticRuleGrant(u16, KeywordGrant),
    StaticModifier(u16, StaticModifier),
    /// Live earlier-layer predicate gates an ordinary creature-group bonus.
    ConditionalCreatureModifier {
        condition: StatValue,
        modifier: StaticModifier,
    },
    /// Live layer-7c bonus while the exact granting source meets its counter threshold.
    SourceCounterBonus {
        kind: CounterKind,
        minimum: u64,
        filter: CreatureFilter,
        stats: Stats,
    },
    ScalingModifier(u16, ScalingModifier),
    EnchantedModifier(u16, EnchantedModifier),
    CounterKeyword(u16, CounterKind, Keywords),
    /// Other controlled creatures with this counter have the keyword.
    CounterKeywordOther(CounterKind, Keywords),
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
    /// Generic reduction for plotting a card from hand.
    PlotCostReduction(u32),
    /// Generic reduction for equip activations targeting this exact permanent.
    EquipTargetReduction(u32),
    /// Optional replacement of the first equip activation's whole cost.
    FirstEquipZeroCost {
        condition: StatValue,
    },
    TargetCostReduction(u16, TargetSpellReduction),
    CombatRestriction(u16, CombatRestriction),
    /// Source attacks every combat when independently able to attack.
    AttacksEachCombat,
    /// Flying creatures cannot attack this controller or block their creatures.
    FlyingCombatRestriction,
    /// The source's evasion ability excludes blockers with an effective subtype.
    CannotBeBlockedBySubtype(&'static str),
    /// Rules restriction evaluated after all derived power modifications.
    CannotBeBlockedIfSourcePowerAtMost(i128),
    CannotBeBlockedByCreaturesWithLessPower,
    /// Controlled creatures meeting either live stat bound cannot be blocked.
    ControlledCreaturesCannotBeBlockedIfPowerOrToughnessAtMost(i128),
    EntryTriggerSuppression(u16, u16),
    OpponentEntryTapped(u16, u16),
    ControlledEntryUntapped,
    StaticCastRestriction(u16, u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UntapRestriction {
    All,
    ControllerStep,
    ControllerStepUnless(StatValue),
}

/// One printed Aura effect, shared across its earlier characteristic layers.
#[derive(Clone, Copy, Debug)]
pub(crate) enum PermanentForm {
    Creature,
    Land,
    /// Add creature type without removing other card or creature types.
    AddCreature,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct AttachedCharacteristics {
    pub name: Option<&'static str>,
    pub form: PermanentForm,
    pub creature_types: &'static [&'static str],
    pub colors: &'static [ChoiceColor],
    pub base: Option<(i128, i128)>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LandManaRecipients {
    YouControl,
    SourceChosenName,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AnimationDuration {
    EndOfTurn,
    Permanent,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AnimationCreatureTypes {
    Unchanged,
    Named(&'static [&'static str]),
    All,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AnimationSpec {
    pub artifact: bool,
    pub creature_types: AnimationCreatureTypes,
    pub base: Option<(i128, i128)>,
    pub keywords: Keywords,
    pub duration: AnimationDuration,
}
