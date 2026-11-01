//! Deterministic in-process Magic engine. Consumers need only this crate.
mod catalogue;
mod ffi;
pub use cards::{
    BasicLandType, CREATURE_TYPES, Card, CardInfo, ChoiceColor, CounterKind, Counters,
    CreatureType, EffectChoiceKind, EmblemInfo, EntryChoiceKind, EntryChoiceValue, GiftKind,
    Keywords, LiveStatCondition, LiveStatRecipients, LiveStatValue, LiveStaticEffect, Mana,
    ManaCost, ManaRetention, MilledDestination, MilledFilter, PlayerCounterKind,
    PlayerCounterSplit, RecoveryFilter, SearchDestination, SearchFilter, TokenInfo,
    is_creature_type,
};
pub use catalogue::CATALOGUE_VERSION;
use rustmage_core::cards;
pub use rustmage_core::engine::{
    AbilityAvailabilityView, AbilityResolutionView, AbilitySourceTimestampView, Action,
    ActivationCostRoute, AdditionalCostView, AdventurePermission, AnimationView,
    AttachmentTriggerView, AttackDeclaration, AttackPaymentView, AuraAttachmentView, AuraEntryView,
    BaseStatsEffectView, BorrowedAbilityView, BoundTarget, CardId, CascadeView, CastCostClause,
    CastCostRow, CastCostTransaction, CastCreatureSubjectsView, CastEntryCounterView, CastFace,
    CastInfo, CastMethod, CastPaymentEvent, CasualtyDeclaration, CasualtyGrant,
    CombatDamagePlayerView, CombatDefender, Config, ConniveChoiceView, ConniverView,
    ControlledPlayer, ConvokeChoiceView, ConvokeContribution, ConvokePaymentView, CopyDeadline,
    CopyEvent, CopyExceptions, CopyLayerView, CopyOverrides, CopyRecipe, CopyTypes,
    CounterPlacementHistoryView, CreatedTokenChoiceView, CreatureCombatEventView,
    CreatureComparisonFacts, CreatureDamageHistoryView, CreatureTypeChoiceView, CrewChoiceView,
    CrewContext, CrewHistory, DamageAssignmentRange, DamageEvent, DamageEventOrderView,
    DamageFacts, DamagePreventionKind, DamagePreventionView, DamageReceivedView, DamageSource,
    DamageTriggerView, DeckRules, DelayedReturnView, DelayedSacrificeView, DelayedWarpExileView,
    DiscardTriggerView, DrawReplacementView, DrawTriggerView, EarthbendReturnView,
    EffectChoiceView, EndureChoiceView, EntryChoiceView, EntryLifeSelection, EntryLifeView,
    EntryTapEffect, EntryTapKind, EntryTapView, EntryTimestampObjectView, EntryTimestampView,
    EntryTransformChoiceView, EquipmentAttachmentView, Error, EventObjectFacts, EventObjectView,
    EvidenceEventView, EvidenceView, ExileDeathReplacementView, ExileLink, ExilePlayExpiry,
    ExilePlayPermission, ExploreChoiceView, ExploreEventView, ExplorerView, ExtraTurnView,
    FaceDownPermanentView, FaceUpPaymentView, FaceView, ForageRoute, ForageView, Game, GiftContext,
    GiftDeclaration, GrantedAbilityView, GrantedStaticRule, GrantedSubtypes,
    GraveyardBorrowedAbilityView, GraveyardCastKind, GraveyardCastPermissionView, GraveyardTypeUse,
    HandChoiceKind, HandChoiceView, HandDeploymentView, HandLibrarySearchView, HandSizeSourceView,
    HandSizeTimelineView, ImproviseChoiceView, KeepTypesView, KeywordChoiceView,
    KeywordEffectOrigin, KeywordEffectView, LegalActionSpace, LegendGroupView, LibraryCastLookView,
    LibraryChoiceView, LibraryDestination, LibraryPile, LibraryPilesView, LibraryRevealEvent,
    LibrarySelectionView, LifeTriggerView, LinkedExileView, LiveStaticSourceView,
    ManaAnnouncementView, ManaCopySource, ManaLotSpendRange, ManaLotView, ManaPaymentDomain,
    ManaRestrictionView, ManaRollbackView, ManaSpellBonusView, ManaValueParity, ManifestChoiceView,
    ManifestDreadEvent, MilledSelectionView, MiracleChoiceView, MobilizeChoiceView,
    ModalHistoryView, ModeChoiceView, NextTurnControlExpiry, OBSERVATION_SCHEMA, ObjectRef,
    ObjectView, Observation, OptionalSearchSequence, Outcome, OutsideEntryDestination,
    OutsideEntryProviderView, OutsideEntryReplacementView, OutsideEntrySelfEffectView,
    OutsideGameConfig, OutsideGameMode, ParadigmView, PaymentRequestView, PendingPaymentView,
    Player, PlayerCastGrantsView, PlayerControlView, PlayerCounterAmountView,
    PlayerCounterPlacementView, PlayerView, PlotPaymentView, PlottedCard, PowerUpUseView,
    PreparationView, ProliferateReplacementView, RecoveryChoiceView, ReflexiveSubjectView,
    ReplacementChoiceView, ReplacementEffectView, ResolutionFreeCasts, ResolutionPaidCast,
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
    TemporaryCopyView, TokenGroupView, TriggerSourceMoveView, TriggerUse, TurnFactsView,
    TurnScheduleView, UntilNextTurnStatsView, WarpedCard, WaterbendChoiceView, XChoiceView,
    XInterval, Zone, ZoneChange, ZoneReplacementChoiceView, ZoneReplacementKind, ZoneRevealView,
    ZoneSearchView,
};
pub use rustmage_core::policy;
pub use rustmage_core::protocol;
pub use rustmage_core::{CapacityExceeded, CardName, PermanentType};

/// Immutable slice-like view returned by `Card::ALL`.
pub use rustmage_core::cards::generated::SupportedCards;
