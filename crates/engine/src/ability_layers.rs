//! Immutable static-effect provenance; shared fragments retain one source identity.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum AbilityLayer {
    Control = 2,
    Text = 3,
    Type = 4,
    #[allow(
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

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug)]
pub struct StaticAbilityGroup {
    /// Stable source-local ability identity; legacy projections use printed lines.
    #[allow(
        dead_code,
        reason = "Immutable origin identity is retained for typed definition migration and diagnostics"
    )]
    pub line: u16,
    pub first_layer: AbilityLayer,
    pub fragments: &'static [StaticAbilityFragment],
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActivationReductionAmount {
    Fixed(u32),
    /// Sample this discount provider's live power before the cost is locked.
    ProviderPower,
}

/// Qualifications belong to the ability's actual permanent source, not the
/// immutable card which supplied a granted program.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdditionalTriggerScope {
    /// A triggered ability of a permanent this provider's controller controls.
    ControlledPermanent,
    ControlledSubtype(&'static str),
    ControlledCreatureSubtype(&'static str),
    ControlledLegendaryCreature,
    OtherControlledSubtype(&'static str),
    /// Abilities of another creature with this provider's active entry choice.
    OtherControlledCreatureOfChosenType,
    EquippedCreature,
    /// Abilities of this equipped source or of any Equipment attached to it.
    SourceAndAttachedEquipment,
    /// A land or this subtype entering caused a controlled permanent's trigger.
    ControlledLandOrSubtypeEntry(&'static str),
    /// Any permanent entering caused a controlled permanent's ability to trigger.
    ControlledPermanentEntry,
    /// Grant the bearer a multiplier for death triggers of itself and emblems it owns.
    EquippedCreatureAndOwnedEmblemDeath,
    /// Only abilities triggered by a creature attacking, never other events.
    ControlledPermanentAttack,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug)]
#[allow(
    dead_code,
    reason = "Complete immutable typed origin payloads are preserved while legacy fast tables supply some values; do not discard during direct-definition migration"
)]
pub enum StaticAbilityFragment {
    /// Qualifies this entire immutable static ability group by its source's
    /// active as-entry choice. Checked definitions restrict this to integrated
    /// consumers. An in-place copy has no inherited choice.
    EntryMode(u16),
    /// A printed activated template present only in this source's chosen mode.
    SourceEntryModeActivation {
        mode: u16,
        ability: u16,
    },
    /// Cast the exact card exiled by this source's linked hand-reveal ability at threshold.
    CastLinkedExileAtThreshold,
    /// Initiates its controller's persistent speed designation as a state-based action.
    StartYourEngines,
    /// No player can draw cards while this rule ability is active.
    CantDrawCards,
    LinkedExileFreeCastBySubtypeUnion(&'static [&'static str]),
    LinkedExileCreatureCounterCasting(u16),
    /// Cast owned cards linked to this permanent using ordinary mana costs.
    LinkedExilePaidCast {
        filter: CastFilter,
        entry_counter: Option<CounterKind>,
    },
    /// Use live toughness rather than power when paying Crew or Saddle costs.
    CrewSaddleWithToughness,
    /// A layer-six activated-ability template granted to this source at max speed.
    SourceMaxSpeedActivation(u16),
    /// Generic increase for activations whose actual source has this chosen name.
    NamedActivationGenericIncrease(u32),
    /// During own turn, ignore spent exhaust use before the first exhaust activation.
    ReuseExhaustBeforeFirstActivationOwnTurn,
    /// Generic reduction for exhaust abilities of other controlled permanents.
    OtherControlledExhaustGenericReduction(u32),
    ArtifactTokenActivationReduction(u32),
    /// Reduce mana for activations of matching battlefield permanents you control.
    ControlledActivationReduction {
        kinds: u16,
        amount: ActivationReductionAmount,
        leave_one_mana: bool,
    },
    ControlledLandTapManaBonus(Mana),
    /// Add one {C} when a controlled land's tap ability produces {C}.
    ControlledLandColorlessTapManaBonus,
    /// Add one mana of a type actually produced by a controlled artifact token tapped for mana.
    ControlledArtifactTokenTapManaBonus,
    AttachedChosenColor,
    AttachmentManaBonus(ManaProduction),
    BasicLandTapManaBonus(u16),
    /// Multiply the original output when the controller taps a basic land for mana.
    ControlledBasicLandManaMultiplier(u16),
    /// Controlled instant and sorcery spells have split second.
    ControlledInstantSorcerySplitSecond,
    /// Grant keywords to controlled instant and sorcery spells on the stack.
    ControlledInstantSorceryKeywords(Keywords),
    /// Permanents controlled by this source's controller cannot be sacrificed.
    ControlledPermanentsCannotBeSacrificed,
    /// No player may activate abilities of matching battlefield permanent kinds.
    ActivatedPermanentKindsProhibited(u16),
    /// The exact attached permanent's activated abilities cannot be activated.
    AttachedActivationsProhibited,
    /// The exact attached permanent cannot be turned face up.
    AttachedFaceUpProhibited,
    /// This source's controller may look at opposing face-down creatures.
    LookAtOpponentFaceDownCreatures,
    /// Cards in graveyards cannot be exiled by any player.
    GraveyardCardsCannotBeExiled,
    /// Replace a controlled permanent's move to a graveyard with no move.
    PreventControlledPermanentGraveyardMove,
    /// Replace this controller's life payments with top-library exile when possible.
    ReplaceLifePaymentWithLibraryExile,
    /// During combat no spells or nonmana activated abilities may be started.
    NoSpellsOrNonmanaDuringCombat,
    /// Opponents who declared an attack on this controller or their planeswalker this turn can't cast.
    OpponentsWhoAttackedControllerOrPlaneswalkerCannotCast,

    /// Cast this exact owned graveyard card normally while the live predicate holds.
    /// This grants neither land plays nor flashback's alternative cost/destination.
    SelfGraveyardCastIf(StatValue),

    /// This exact owned graveyard card has a separate ordinary-mana casting route.
    SelfGraveyardCastWithAdditionalCosts {
        life: u32,
        additional: crate::cards::AdditionalCost,
    },

    /// Grants its controller the permanent enduring-story designation at threshold.
    Storied,
    /// This source continuously acquires activations from matching cards in its controller graveyard.
    BorrowGraveyardActivations(&'static str),
    /// Controlled creatures with this counter gain activated abilities from linked exiled creatures.
    GrantLinkedExileCreatureActivationsWithCounter(CounterKind),
    /// This creature can attack despite defender after its controller scries or surveils.
    AttackWithoutDefenderIfScried,
    /// This exact source can attack despite defender while the live condition holds.
    AttackWithoutDefenderIf(StatValue),
    /// Check a live battlefield guard for this permanent's defender permission.
    AttackWithoutDefenderWhen(StepGuard),
    /// Controlled creatures may attack despite defender.
    ControlledAttackWithoutDefender,
    /// This source cannot be attacked while attached to a creature.
    AttachedCreaturePreventsSourceAttack,
    /// Controlled creatures with greater toughness assign combat damage using toughness.
    ControlledCombatDamageByGreaterToughness,
    /// Controlled creatures with greater toughness use it when station resolves.
    ControlledStationWithGreaterToughness,
    /// Attached creature assigns combat damage with toughness only when greater.
    AttachmentCombatToughnessIfGreater,
    /// Add one independent blocking requirement for this creature.
    MustBeBlocked,
    /// This source assigns combat damage using the magnitude of negative power.
    SourceNegativePowerCombatDamage,
    /// Replace each nonexempt controller draw with two draws.
    DoubleDrawExceptFirst,
    /// Add one scry before each explore by a creature this player controls.
    ScryBeforeControlledCreatureExplores,
    /// Double each controller draw, including their normal draw-step draw, while true.
    ConditionalDoubleDraw(StatValue),
    /// Add one to a controller draw instruction while their hand has at most one card.
    AdditionalDrawWithAtMostOneCard,
    /// Increase opponents' noncreature spell costs by generic mana.
    OpponentNoncreatureSpellTax(u32),
    /// Generic cost increase on opposing casts from a graveyard or exile.
    OpponentGraveyardOrExileSpellTax(u32),
    /// Each player can cast at most this many spells per turn.
    SpellCastLimit(u64),
    /// Each opponent spell targeting this permanent costs an additional life payment.
    OpponentSpellsTargetingSourceLifeCost(u32),
    /// Increase both players' spell costs for the type chosen as this source entered.
    ChosenCardTypeSpellTax(u32),
    /// Generic cost per creature declared attacking this source's controller.
    AttackControllerTax {
        amount: u32,
        condition: StatValue,
    },
    /// Generic cost per creature declared attacking this source's controller or their planeswalker.
    AttackControllerOrPlaneswalkerTax {
        amount: u32,
        condition: StatValue,
    },
    /// Generic cost per declared blocker while this permanent is attacking.
    BlockTaxWhileAttacking(u32),
    AdditionalTriggers {
        scope: AdditionalTriggerScope,
        condition: StatValue,
    },
    FreeHandCast,
    /// An alternative mana cost only; the card needs an independent legal origin.
    FreeSpellCast(CastFilter),
    /// Both players may take this alternative for their first spell of their turn.
    FreeFirstSpellOnOwnTurn,
    GrantHandWarp {
        filter: CastFilter,
        cost: ManaCost,
    },
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
    /// Nonland cards on top of this controller's library may be plotted, including for their mana cost.
    PlotFromLibraryTop,
    CastCreatureFromLibraryTop,
    /// An ordinary cast from the current top, using the chosen spell face.
    CastFromLibraryTop(CastFilter),
    /// Each controller may use this exact ability for one library-top spell per turn.
    CastInstantSorceryFromLibraryTopOnceEachTurn,
    /// Ordinary land plays and spell casts after this exact source attacked.
    PlayFromLibraryTopAfterAttacking,
    CastSubtypesFromLibraryTopWithCounter(&'static [&'static str]),
    PlayLandFromLibraryTop,
    PlayLandFromGraveyard,
    PlayFromGraveyardDuringOwnTurn,
    /// Eligible nonland faces in your graveyard have Mayhem for their mana cost.
    GrantManaCostMayhem,
    /// Cast a matching spell from your graveyard with an additional life cost.
    /// The permanent enters with the specified counter if cast this way.
    CastFromGraveyardWithLife {
        filter: CastFilter,
        life: u32,
        entry_counter: CounterKind,
    },
    /// Cast a matching spell from your graveyard on your turn with additional life payment.
    CastFromGraveyardWithLifeDuringOwnTurn {
        filter: CastFilter,
        life: u32,
    },
    SpendAnyManaForCreatures,
    /// Actual mana can meet WUBRG requirements of controlled creatures' activations.
    SpendAnyColorForCreatureActivations,
    SpellImprovise(CastFilter),
    /// Controlled matching spells have delve while this provider's rule is available.
    SpellDelve(CastFilter),
    GraveyardPermanentTypes,
    /// Play qualifying creature spells from your graveyard during your turn.
    GraveyardSmallCreaturesWithFinality,
    GrantGraveyardCreatureSneak {
        minimum_class_level: u8,
        cost: ManaCost,
    },
    CastGraveyardCreaturesUsingSneak {
        minimum_class_level: u8,
    },
    ForeignStashPlay,
    ControlAttached,
    AttachedCharacteristics(&'static AttachedCharacteristics),
    ChosenCreatureType,
    /// Layer-four type grant supplied by an attached permanent.
    AttachedAllCreatureTypes,
    /// Add this source’s type to controlled creatures, optionally of a required subtype.
    ControlledCreatureSubtype {
        grant: CreatureSubtypeGrant,
        required_subtype: Option<&'static str>,
    },
    /// Add this subtype to controlled creatures owned by another player.
    ControlledForeignOwnedCreatureSubtype(&'static str),
    /// This source adds artifact and creature during another player’s turn.
    SourceOffTurnCreature,
    /// This land is the chosen basic land type (CR305.7).
    ChosenBasicLandType,
    OtherSubtypeEntryCounters(&'static str),
    /// A fixed addition from a provider already present before the entry event.
    ControlledSubtypeEntryCounters {
        subtype: &'static str,
        kind: CounterKind,
        amount: u64,
    },
    /// Controlled creatures enter with an additional +1/+1 counter per land
    /// that previously entered under this provider controller this turn.
    ControlledCreatureEntryCountersForLandsEntered,
    /// Add a live quantity of counters to other controlled creatures as they enter.
    OtherControlledCreatureEntryCountersByStat {
        kind: CounterKind,
        amount: StatValue,
    },
    /// Additional counters on other controlled entrants, selected by effective mana value.
    OtherControlledEntryCountersByManaValue {
        filter: ReplacementFilter,
        kind: CounterKind,
        maximum: u64,
        at_most: u16,
        above: u16,
    },
    ChosenModifier(ChosenModifier),
    /// Grant keywords to creatures of the source's chosen creature type.
    ChosenTypeKeywords(Keywords),
    /// Sample an earlier-layer quantity for each controlled creature's modifier.
    ControlledCreatureScaling {
        subtype: Option<&'static str>,
        value: StatValue,
        stats: Stats,
    },
    /// Grant keywords to controlled battlefield permanents matching a live kind mask.
    /// Any creature attacking an opponent player of this source controller.
    OpponentPlayerAttackerKeywords(Keywords),
    ControlledPermanentKeywords {
        kinds: u16,
        keywords: Keywords,
    },
    /// Live grant to equipped creatures, independent of the Equipment controller.
    ControlledEquippedCreatureKeywords(Keywords),
    ConditionalEquippedCreatureKeywords {
        condition: StatValue,
        keywords: Keywords,
    },
    /// Live grant to tapped controlled creatures, including virtual payment taps.
    ControlledTappedKeywords {
        other: bool,
        keywords: Keywords,
    },
    /// Live grant to other tapped legendary creatures while the provider qualifies.
    ControlledOtherTappedLegendaryCreatureKeywords {
        condition: StatValue,
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
    /// Each other controlled permanent of this subtype has an instance of Riot.
    GrantRiotToOtherControlledSubtype(&'static str),
    /// Live layer-six grant; virtual taps and control/subtype changes requalify recipients.
    ControlledUntappedSubtypeKeywords {
        subtype: &'static str,
        keywords: Keywords,
    },
    PlayerHexproof,
    PlayerProtectionFromOpponents,
    /// This permanent has protection from sources with the specified effective subtype.
    ProtectionFromSubtype(&'static str),
    ActivateControlledCreaturesAsHaste,
    PlayerHexproofIfSourceCounter {
        kind: CounterKind,
        minimum: u64,
    },
    /// Real immutable trigger template, present only on the harnessed source.
    HarnessedTrigger(u16),
    CantLoseGameAndOpponentsCantWin,
    /// The controller does not lose the state-based action for life at or below zero.
    CantLoseFromNonpositiveLife,
    QualifiedHexproof(SourceCharacteristicFilter),
    ControlledSpellsUncounterable(u16),
    /// A rule restriction on explicit counters, independent of stack target legality.
    StackItemsCannotBeCountered,
    /// Colored legendary spells you cast have this Web-slinging alternative.
    ColoredLegendaryWebSlinging(ManaCost),
    /// Each instance grants a distinct optional paired-tap copy cost.
    NoncreatureConspire,
    ConvokeCreatureSpells,
    ControlledSpellsCasualty {
        kinds: u16,
        minimum: u16,
    },
    LifeGainAdditional(u16),
    LifeGainDouble,
    OpponentLifeLossDoubleOnOwnTurn,
    /// Add to an opponent's positive mill event.
    OpponentMillAdditional(u16),
    /// The controller's first positive batch of coin flips comes up heads and is won.
    FirstCoinFlipsHeads,
    CantGainLife,
    EnchantedPlayerCantGainLife,
    DamageCantBePrevented,
    /// Combat damage from any source ignores prevention.
    CombatDamageCantBePrevented,
    /// This exact permanent keeps marked damage as the cleanup step removes other damage.
    RetainDamageDuringCleanup,
    /// This ability's controller keeps mana of the specified color at boundaries.
    RetainManaColor(ChoiceColor),
    NoMaximumHandSize,
    MaximumHandSize(u16),
    /// While the controller has at least this many card types in their graveyard,
    /// the opponent's maximum is the base minus the live count, floored at zero.
    OpponentMaximumHandSizeByGraveyardTypes {
        base: u16,
        minimum: u16,
    },
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
    RemoveAttachedKeywords(Keywords),
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
    /// Include this permanent in the turn-based action on another player's turn.
    UntapDuringOtherUntapSteps,
    /// Include creatures this source's controller controls in other players' untap steps.
    UntapControlledCreaturesDuringOtherUntapSteps,
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
        form: SourceCreatureForm,
        subtypes: &'static [&'static str],
        keywords: Keywords,
    },
    /// On celebration, replace creature subtypes and grant a battlefield activation
    /// and keywords to this source. Pair its base setter in the same type-layer group.
    SourceCelebrationCreatureForm {
        subtypes: &'static [&'static str],
        keywords: Keywords,
        activation: u16,
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
    /// For each Aura or Equipment attached to the same recipient.
    AttachmentStatsPerAttachment(Stats),
    AttachmentTurnStats(Stats),
    AttachmentKeywords(Keywords),
    AttachmentEntryTurnKeywords(Keywords),
    AttachmentAttackingKeywords(Keywords),
    AttachedAttackerMustBeBlocked,
    AttachmentRule(Keywords),
    /// Choose the grant by the attachment controller's turn.
    AttachmentTurnChoiceKeywords {
        own_turn: Keywords,
        other_turn: Keywords,
    },
    AttachedTurnKeywords(Keywords),
    /// Stable index into the granting definition's immutable trigger array.
    AttachmentTrigger(u16),
    /// The attached permanent's controller may privately view their library top.
    AttachmentLookAtLibraryTop,
    /// Each matching creature owns the indexed granted trigger template.
    /// Scope is evaluated in layer six. Cast triggers sample the committed
    /// battlefield; departure triggers retain the pre-move instance.
    ControlledCreatureTrigger {
        ability: u16,
        other: bool,
        nontoken: bool,
    },
    /// A real Ward instance granted by a live conditional static ability.
    ConditionalWard {
        condition: StatValue,
        filter: CountFilter,
        ability: u16,
    },
    /// Grant the indexed ward to controlled permanents with at least one counter of any kind.
    CounterBearingPermanentWard {
        condition: StatValue,
        ability: u16,
    },
    /// During the provider's turn, grant both abilities to it and controlled creatures of these kinds.
    OwnTurnSourceAndKindCreatureWard {
        kinds: u16,
        keywords: Keywords,
        ward: u16,
    },
    /// Grant a creature the indexed ordinary activation template. Tapping the
    /// distinct attachment is an additional cost only when printed explicitly.
    AttachmentActivation {
        ability: u16,
        tap_attachment: bool,
    },
    /// Grants an independent tap mana ability to the attached permanent.
    AttachmentMana(ManaProduction),
    AttachmentSacrificeMana(ManaProduction),
    LandMana {
        recipients: LandManaRecipients,
        production: ManaProduction,
    },
    /// Grant a tap mana ability to each other permanent its controller controls.
    OtherControlledPermanentTapMana(ManaProduction),
    /// Grant a tap mana ability to controlled permanents of the indicated card types.
    ControlledPermanentTapMana {
        kinds: u16,
        production: ManaProduction,
    },
    /// Grant a tap mana ability to controlled permanents with this effective subtype.
    ControlledSubtypeTapMana {
        subtype: &'static str,
        production: ManaProduction,
    },
    /// Other controlled permanents of this subtype gain the indexed mana ability.
    OtherControlledSubtypeTapManaActivation {
        subtype: &'static str,
        ability: u16,
    },
    /// In layer four add Food to controlled artifacts. In layer six grant them
    /// this indexed activation template as part of the same continuous effect.
    ControlledArtifactFood(u16),
    /// In layer four add the artifact type and Food subtype to every other
    /// creature. In layer six grant each the indexed activation template.
    OtherCreaturesFood(u16),
    /// In layer six grant this indexed activation to controlled artifacts.
    ControlledArtifactActivation(u16),
    /// In layer six, grant this indexed nonmana activation to controlled nontoken creatures.
    ControlledNontokenCreatureActivation(u16),
    KeywordGrant(u16, KeywordGrant),
    StaticRuleGrant(u16, KeywordGrant),
    ControlledCreatureCounterRule {
        kind: Option<CounterKind>,
        keywords: Keywords,
    },
    StaticModifier(u16, StaticModifier),
    /// One layer-7c effect applies once to the union of its live creature groups.
    CreatureGroupBonus {
        filters: &'static [CreatureFilter],
        stats: Stats,
    },
    /// Scale one live layer-7c effect across the union of its creature groups.
    CreatureGroupScaling {
        filters: &'static [CreatureFilter],
        value: StatValue,
        stats: Stats,
    },
    /// A layer-7c bonus to the source controller's current legendary creatures.
    ControlledLegendaryCreatureBonus(Stats),
    /// A layer-7c bonus to controlled creatures with every listed card type.
    ControlledCreatureKindBonus {
        kinds: u16,
        stats: Stats,
    },
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
    RoomUnlockReduction(u32),
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
    /// Creatures enchanted by an Aura you control cannot attack you or your planeswalkers.
    AuraEnchantedCantAttackController,
    /// The source's evasion ability excludes blockers with an effective subtype.
    CannotBeBlockedBySubtype(&'static str),
    ControlledSubtypeCannotBeBlockedByKeyword {
        subtype: &'static str,
        blocker_keyword: Keywords,
    },
    ControlledSubtypeIgnoresLegendRule(&'static str),
    /// Rules restriction evaluated after all derived power modifications.
    CannotBeBlockedIfSourcePowerAtMost(i128),
    CannotBeBlockedByCreaturesWithLessPower,
    CannotBeBlockedByCreaturesWithPowerAtLeast(i128),
    /// This permanent can block only attacking creatures with flying.
    CanBlockOnlyFlying,
    /// Controlled creatures meeting either live stat bound cannot be blocked.
    ControlledCreaturesCannotBeBlockedIfPowerOrToughnessAtMost(i128),
    EntryTriggerSuppression(u16, u16),
    OpponentEntryTapped(u16, u16),
    ControlledEntryUntapped,
    StaticCastRestriction(u16, u8),
}

/// An immutable named creature subtype or the exact source’s as-entry choice.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreatureSubtypeGrant {
    Named(&'static str),
    Chosen,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UntapRestriction {
    All,
    ControllerStep,
    ControllerStepUnless(StatValue),
}

/// One printed Aura effect, shared across its earlier characteristic layers.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug)]
pub enum PermanentForm {
    /// Replace only creature subtypes while retaining all card types and, for
    /// an empty color list, the existing colors.
    Preserve,
    /// Retain card types but set colors, including an explicit colorless result.
    PreserveWithColors,
    Creature,
    Land,
    /// Replace card types with artifact and add Food, keeping artifact subtypes.
    FoodArtifact,
    /// Add creature type without removing other card or creature types.
    AddCreature,
    /// Add creature subtypes without changing card types or colors.
    AddCreatureSubtypes,
    /// Add the given card types and the named creature subtypes.
    AddCardTypes(u16),
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug)]
pub enum SourceCreatureForm {
    /// Set the card type and creature subtypes for a plain creature form.
    Creature,
    /// Becoming an artifact creature retains other card types (CR 205.1b).
    ArtifactCreature,
    /// Add only the creature type, as a Station threshold does (CR 721.2b).
    AddCreature,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug)]
pub struct AttachedCharacteristics {
    pub name: Option<&'static str>,
    pub form: PermanentForm,
    pub creature_types: &'static [&'static str],
    pub colors: &'static [ChoiceColor],
    pub base: Option<(i128, i128)>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LandManaRecipients {
    YouControl,
    SourceChosenName,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationDuration {
    EndOfTurn,
    Permanent,
}
/// Layer-four and layer-five additions to the same exact permanent.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CharacteristicAddition {
    pub card_types: u16,
    pub creature_types: &'static [&'static str],
    pub colors: u8,
    pub duration: AnimationDuration,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationCreatureTypes {
    Unchanged,
    Named(&'static [&'static str]),
    /// Add creature types while retaining existing subtypes.
    Additional(&'static [&'static str]),
    All,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationSpec {
    pub artifact: bool,
    pub creature_types: AnimationCreatureTypes,
    pub base: Option<(i128, i128)>,
    pub keywords: Keywords,
    pub duration: AnimationDuration,
}
