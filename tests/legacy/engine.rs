use super::*;
use crate::PermanentType;
use crate::capacity::{CapacityExt, CapacitySignal};
pub use crate::cards::CopyExceptions;
pub use crate::cards::GraveyardCastKind;
use crate::cards::*;
use crate::cards::{
    ActivatedAbility, ActivationCost, ActivationGuard, ActivationOrigin, AdditionalCost,
    AmountValue, BattlefieldFilter, CastRelation, CostTurn, CountFilter, CountZone, CounterAmounts,
    CounterKind, CounterSubject, Counters, CreatureFilter, DamageQualification, DamageSubject,
    Effect, EffectChoiceKind, EmblemInfo, EntryChoiceKind, EntryChoiceValue, EntryCounterAmount,
    EntryPredicate, EntryTapped, InitialCounters, Instruction, ManaCost, ManaProduction,
    MillAmount, MilledCount, MilledDestination, MilledFilter, MilledSelectionSpec, NamedCounters,
    PlayerSet, RecoveryFilter, ResolvedStatValue, ReturnKind, SacrificeFilter, SearchSpec,
    SpellMode, StatExpr, StatValue, Stats, StepGuard, StepValue, Subject, TargetGroupConstraint,
    TargetPlan, TargetPredicate, TargetRule, TargetSpec, TargetZone, TriggerCondition, TriggerStep,
    payments,
};
use crate::rng::Rng;
use crate::shared_vec::SharedVec;
use crate::{Card, Keywords, Mana, TokenInfo};
pub use ability_evaluation::{AbilityAvailabilityView, GrantedAbilityView, GrantedStaticRule};
pub use ability_resolutions::AbilityResolutionView;
pub use ability_timeline::{AbilitySourceTimestampView, KeywordEffectOrigin, KeywordEffectView};
pub use additional_costs::AdditionalCostView;
pub use adventure::{AdventurePermission, SpellCharacteristicsView};
pub use base_stats::{BaseStatsEffectView, ResolvedStatSubjectView, UntilNextTurnStatsView};
use borrowed_activations::Descriptor as BorrowedDescriptor;
pub use borrowed_activations::{BorrowedAbilityView, GraveyardBorrowedAbilityView};
pub use card_quantities::SourceCounterQuantityView;
pub use cascade::CascadeView;
pub use cast_costs::{CastCostClause, CastCostRow, CastCostTransaction};
pub use cast_entry_counters::CastEntryCounterView;
pub use cast_origins::{GraveyardTypeUse, ResolutionFreeCasts, ResolutionPaidCast};
pub use cast_payments::{CastPaymentEvent, ManaPaymentDomain};
pub use cast_permissions::GraveyardCastPermissionView;
pub use cast_rollback_mana::ManaRollbackView;
pub use cast_subjects::CastCreatureSubjectsView;
pub use casualty::{CasualtyDeclaration, CasualtyGrant};
pub use characteristic_layers::AnimationView;
pub use combat_allocations::CombatDamagePlayerView;
pub use combat_restrictions::AttackPaymentView;
pub use connive::{ConniveChoiceView, ConniverView};
pub use convoke::{ConvokeChoiceView, ConvokeContribution, ConvokePaymentView};
use copies::CopyFaces;
pub use copies::CopyRecipe;
pub use counter_placement_history::CounterPlacementHistoryView;
pub use creature_type_tokens::CreatureTypeChoiceView;
pub use crew::{CrewChoiceView, CrewContext, CrewHistory};
pub use death_exile::ExileDeathReplacementView;
pub use discard_events::DiscardTriggerView;
pub use draw_events::DrawTriggerView;
pub use earthbend::EarthbendReturnView;
pub use entries::{
    AuraAttachmentView, AuraEntryView, EntryLifeSelection, EntryLifeView, EntryTimestampObjectView,
    EntryTimestampView,
};
pub use entry_choices::EntryChoiceView;
pub use entry_tap::{EntryTapEffect, EntryTapKind, EntryTapView};
pub use event_counter_core::CreatureDamageHistoryView;
pub use event_objects::{CreatureComparisonFacts, EventObjectFacts, EventObjectView};
pub use evidence::{EvidenceEventView, EvidenceView};
pub use exile::{ExilePlayExpiry, ExilePlayPermission};
pub use explore::{ExploreChoiceView, ExploreEventView, ExplorerView, RevealedLibraryCard};
pub use faces::FaceView;
pub use forage::{ForageRoute, ForageView};
use funding::FundingGoal;
pub use gift::{GiftContext, GiftDeclaration};
pub use hand_choices::{HandChoiceKind, HandChoiceView};
pub use hand_library_search::HandLibrarySearchView;
pub use hand_size::{HandSizeSourceView, HandSizeTimelineView};
pub use improvise::ImproviseChoiceView;
pub use library_cast_look::LibraryCastLookView;
pub use library_hand_exchange::HandDeploymentView;
pub use library_selection::{EntryTransformChoiceView, LibrarySelectionView};
pub use library_top::LibraryRevealEvent;
pub use linked_choices::{KeepTypesView, LibraryPile, LibraryPilesView, LinkedExileView};
pub use live_static_stats::LiveStaticSourceView;
pub use mana_announcements::ManaAnnouncementView;
pub use mana_lots::{
    ManaCopySource, ManaLotSpendRange, ManaLotView, ManaRestrictionView, ManaSpellBonusView,
};
pub use mana_value_parity::ManaValueParity;
pub use manifest::{
    FaceDownPermanentView, FaceUpPaymentView, ManifestChoiceView, ManifestDreadEvent,
};
pub use milled_selection::MilledSelectionView;
pub use miracle::{DrawReplacementView, MiracleChoiceView};
pub use mobilize::{DelayedSacrificeView, MobilizeChoiceView};
pub use modal_history::ModalHistoryView;
pub use outside_choices::{
    OutsideEntryDestination, OutsideEntryProviderView, OutsideEntryReplacementView,
    OutsideEntrySelfEffectView,
};
pub use paradigm::{ParadigmView, ResolvedSpellNameView};
pub use player_control::{ControlledPlayer, PlayerControlView};
pub use plot::{PlotPaymentView, PlottedCard};
pub use power_up::PowerUpUseView;
pub use prepared::PreparationView;
use programs::Resolution;
pub use replacements::{
    CreatedTokenChoiceView, DamageEventOrderView, DamagePreventionKind, DamagePreventionView,
    PlayerCounterPlacementView, ProliferateReplacementView, ReplacementChoiceView,
    ReplacementEffectView, SelfPlayerCounterReplacement, TokenGroupView,
};
pub use resolution_mana::ResolutionPaymentView;
pub use return_ability_modification::ResolvedAbilityModificationView;
pub use return_attachment::EquipmentAttachmentView;
pub use return_links::{DelayedReturnView, ReturnCardView, ReturnLinkView};
pub use roles::{RoleGroupView, RoleTimestampView};
pub use rustmage_core::engine::*;
pub use saddle::SaddleChoiceView;
pub use search::OptionalSearchSequence;
pub use self_granted::{SelfGrantedAbilityKind, SelfGrantedAbilityView};
pub use self_granted_activations::SelfGrantedActivationContext;
use serde::{Deserialize, Serialize};
pub use source_linked_effects::{SourceLinkedEffectKind, SourceLinkedEffectView};
pub use source_quantities::SourceQuantityView;
pub use stack_copies::{CopyEvent, RetargetView};
pub use subtypes::GrantedSubtypes;
pub use target_allocations::{TargetAllocation, TargetAmountChoiceView};
pub use target_library::{TargetLibraryMovement, TargetLibraryView};
use targeting::Announcement;
pub use teamwork::TeamworkChoiceView;
pub use temporary_abilities::TemporaryAbilityView;
pub use temporary_copy::{
    CopyDeadline, CopyLayerView, CopyOverrides, CopyTypes, TemporaryCopyView,
};
pub use turn_cast_grants::PlayerCastGrantsView;
pub use turn_facts::TurnFactsView;
pub use turn_schedule::{ExtraTurnView, TurnScheduleView};
pub use warp::{DelayedWarpExileView, WarpedCard};
pub use waterbend::WaterbendChoiceView;
pub use zone_effects::{NextTurnControlExpiry, TemporaryControlView};
pub use zone_replacements::{ZoneReplacementChoiceView, ZoneReplacementKind, ZoneRevealView};
pub use zone_search::ZoneSearchView;
#[path = "engine_ability_order.rs"]
mod ability_order;
#[path = "engine_additional_costs.rs"]
mod additional_costs;
#[cfg(test)]
#[path = "assurance/mod.rs"]
mod assurance;
#[cfg(test)]
#[path = "engine_big_core_tests.rs"]
mod big_core_tests;
#[cfg(test)]
#[path = "engine_big_equipment_tests.rs"]
mod big_equipment_tests;
#[cfg(test)]
#[path = "engine_big_game_tests.rs"]
mod big_game_tests;
#[cfg(test)]
#[path = "engine_big_inherited_tests.rs"]
mod big_inherited_tests;
#[cfg(test)]
#[path = "engine_big_library_tests.rs"]
mod big_library_tests;
#[cfg(test)]
#[path = "engine_big_token_tests.rs"]
mod big_token_tests;
#[path = "engine_card_type_assignment.rs"]
mod card_type_assignment;
#[path = "engine_discard_events.rs"]
mod discard_events;
#[cfg(test)]
#[path = "engine_requested_sapling_tests.rs"]
mod engine_requested_sapling_tests;
#[path = "engine_funding.rs"]
mod funding;
#[path = "engine_manifest.rs"]
mod manifest;
#[path = "engine_outside_choices.rs"]
mod outside_choices;
#[path = "engine_outside_names.rs"]
mod outside_names;
#[path = "engine_player_counter_events.rs"]
mod player_counter_events;
#[path = "engine_replacements.rs"]
mod replacements;
#[cfg(test)]
#[path = "engine_sos_green_colorless_tests.rs"]
mod sos_green_colorless_tests;
#[path = "engine_subtypes.rs"]
mod subtypes;
#[cfg(test)]
#[path = "engine_territory_forge_tests.rs"]
mod territory_forge_tests;
