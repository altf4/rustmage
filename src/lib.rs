//! Deterministic in-process Magic engine. Consumers need only this crate.
mod catalogue;
mod ffi;
pub use cards::{
    BasicLandType, CREATURE_TYPES, Card, CardInfo, ChoiceColor, CounterKind, Counters,
    CreatureType, EffectChoiceKind, EmblemInfo, EntryChoiceKind, EntryChoiceValue, GiftKind,
    Keywords, LiveStatCondition, LiveStatRecipients, LiveStatValue, LiveStaticEffect, Mana,
    ManaCost, ManaRetention, MilledDestination, MilledFilter, NoncreatureCardType,
    PlayerCounterKind, PlayerCounterSplit, RecoveryFilter, SearchDestination, SearchFilter,
    TokenInfo, TurnCallbackEvent, is_creature_type,
};
pub use catalogue::CATALOGUE_VERSION;
use rustmage_core::cards;
pub use rustmage_core::cards::RetainedCopyActivation;
pub use rustmage_core::engine::ChosenCreatureLinkView;
pub use rustmage_core::engine::combat_allocations::PlayerCombatDamageView;
pub use rustmage_core::engine::effect_choices::PowerDifferenceView;
pub use rustmage_core::engine::exhaust::GrantedExhaustUseView;
pub use rustmage_core::engine::replacements::CreatureDamageShieldView;
pub use rustmage_core::engine::return_links::PlayerDamageShieldView;
pub use rustmage_core::engine::temporary_copy::RetainedActivationOrigin;
pub use rustmage_core::engine::{
    AbilityAvailabilityView, AbilityResolutionView, AbilitySourceTimestampView, Action,
    ActivationCostRoute, AdditionalCostView, AdditionalEndStepsView, AdventurePermission,
    AnimationView, AttachmentTriggerView, AttackDeclaration, AttackPaymentView, AuraAttachmentView,
    AuraEntryView, BaseStatsEffectView, BorrowedAbilityView, BoundTarget, CardId, CascadeView,
    CastCostClause, CastCostRow, CastCostTransaction, CastCreatureSubjectsView,
    CastEntryCounterView, CastFace, CastInfo, CastMethod, CastPaymentEvent, CasualtyDeclaration,
    CasualtyGrant, CombatDamagePlayerView, CombatDefender, Config, ConniveChoiceView, ConniverView,
    ConspireDeclaration, ConspireGrant, ControlledPlayer, ConvokeChoiceView, ConvokeContribution,
    ConvokePaymentView, CopyDeadline, CopyEvent, CopyExceptions, CopyLayerView, CopyOverrides,
    CopyRecipe, CopyTypes, CounterPlacementHistoryView, CounterRemovalDomain,
    CreatedTokenChoiceView, CreatureCombatEventView, CreatureComparisonFacts,
    CreatureDamageHistoryView, CreatureEntryHistoryView, CreatureTypeChoiceView, CrewChoiceView,
    CrewContext, CrewHistory, DamageAssignmentRange, DamageEvent, DamageEventOrderView,
    DamageFacts, DamagePreventionKind, DamagePreventionView, DamageReceivedView, DamageSource,
    DamageTriggerView, DeckRules, DeclaredAttackHistoryView, DelayedReturnView,
    DelayedSacrificeView, DelayedTargetTriggerView, DelayedWarpExileView, DiscardTriggerView,
    DrawReplacementView, DrawTriggerView, DreamExileView, EarthbendReturnView, EffectChoiceView,
    EmbalmAbilityView, EndureChoiceView, EntryBatchCopyView, EntryChoiceView, EntryCopyChoiceView,
    EntryDevourChoiceView, EntryExileChoiceView, EntryLifeSelection, EntryLifeView,
    EntryRiotChoiceView, EntryTapEffect, EntryTapKind, EntryTapView, EntryTimestampObjectView,
    EntryTimestampView, EntryTransformChoiceView, EquipmentAttachmentView, Error, EventObjectFacts,
    EventObjectView, EvidenceEventView, EvidenceView, ExertionView, ExileDeathReplacementView,
    ExileLink, ExilePlayExpiry, ExilePlayPermission, ExploreChoiceView, ExploreEventView,
    ExplorerView, ExtraTurnView, FaceDownPermanentView, FaceUpPaymentView, FaceView, ForageRoute,
    ForageView, Game, GiftContext, GiftDeclaration, GrantedAbilityView, GrantedStaticRule,
    GrantedSubtypes, GraveyardBorrowedAbilityView, GraveyardCastKind, GraveyardCastPermissionView,
    GraveyardTypeUse, HandChoiceKind, HandChoiceView, HandDeploymentView, HandLibrarySearchView,
    HandSizeSourceView, HandSizeTimelineView, ImproviseChoiceView, KeepTypesView,
    KeywordChoiceView, KeywordEffectOrigin, KeywordEffectView, LegalActionSpace, LegendGroupView,
    LibraryCastLookView, LibraryChoiceView, LibraryDestination, LibraryPile, LibraryPilesView,
    LibraryRevealEvent, LibrarySelectionView, LifeTriggerView, LinkedExileView,
    LinkedFreeCastApprovalView, LinkedFreeCastUseView, LiveStaticSourceView, MainPhaseCombatView,
    ManaAnnouncementView, ManaCopySource, ManaLotSpendRange, ManaLotView, ManaPaymentDomain,
    ManaRestrictionView, ManaRollbackView, ManaSpellBonusView, ManaTapBonus, ManaValueParity,
    ManifestChoiceView, ManifestDreadEvent, MilledSelectionView, MiracleChoiceView,
    MobilizeChoiceView, ModalHistoryView, ModeChoiceView, NextTurnControlExpiry,
    OBSERVATION_SCHEMA, ObjectRef, ObjectView, Observation, OptionalSearchSequence, Outcome,
    OutsideEntryDestination, OutsideEntryProviderView, OutsideEntryReplacementView,
    OutsideEntrySelfEffectView, OutsideGameConfig, OutsideGameMode, OwnTurnDeadline,
    PaidAdditionalCostView, ParadigmView, PaymentRequestView, PendingPaymentView, Player,
    PlayerCastGrantsView, PlayerControlView, PlayerCounterAmountView, PlayerCounterPlacementView,
    PlayerView, PlotPaymentView, PlottedCard, PowerUpUseView, PreparationView,
    ProliferateReplacementView, RecoveryChoiceView, ReflexiveSubjectView, ReplacementChoiceView,
    ReplacementEffectView, ResolutionCostView, ResolutionFreeCasts, ResolutionPaidCast,
    ResolutionPaymentView, ResolvedAbilityModificationView, ResolvedSpellNameView,
    ResolvedStatSubjectView, RetargetView, ReturnCardView, ReturnLinkView, RevealedLibraryCard,
    RoleGroupView, RoleTimestampView, SaddleChoiceView, SearchChoiceView,
    SearchDestinationChoiceView, SearchDistributionView, SelfGrantedAbilityKind,
    SelfGrantedAbilityView, SelfGrantedActivationContext, SelfPlayerCounterReplacement, Snapshot,
    SourceCounterChoiceView, SourceCounterPaymentView, SourceCounterQuantityView, SourceLessRule,
    SourceLinkedEffectKind, SourceLinkedEffectView, SourceQuantityView, SpellCharacteristicsView,
    SpellCostChoiceView, SpellView, StackKind, StackLink, StackSourceView, Step, Target,
    TargetAllocation, TargetAmountChoiceView, TargetCounterRemovalView, TargetLibraryMovement,
    TargetLibraryView, TeamworkChoiceView, TemporaryAbilityView, TemporaryControlView,
    TemporaryCopyView, TokenGroupView, TriggerSourceMoveView, TriggerUse, TurnCallbackView,
    TurnFactsView, TurnScheduleView, UntilNextTurnStatsView, WarpedCard, WaterbendChoiceView,
    WebSlingingOrigin, XChoiceView, XInterval, Zone, ZoneChange, ZoneReplacementChoiceView,
    ZoneReplacementKind, ZoneReplacementPickView, ZoneRevealView, ZoneSearchView,
};
pub use rustmage_core::engine::{
    CopiedSagaLoreView, DamageDoublerView, LifeGainReplacement, LifeGainReplacementChoice,
    LifeGainReplacementEffect, ManaSplitChoiceView, MeldDepartureOrder, MeldOrderingChoice,
    MeldedPermanentView,
};
pub use rustmage_core::policy;
pub use rustmage_core::protocol;
pub use rustmage_core::{CapacityExceeded, CardName, PermanentType};

/// Immutable slice-like view returned by `Card::ALL`.
pub use rustmage_core::cards::generated::SupportedCards;
