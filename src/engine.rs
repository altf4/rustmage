#[path = "engine_zone_search.rs"]
mod zone_search;
pub use zone_search::ZoneSearchView;
#[path = "engine_library_cast_look.rs"]
mod library_cast_look;
pub use library_cast_look::LibraryCastLookView;
#[path = "engine_player_control.rs"]
mod player_control;
pub use player_control::{ControlledPlayer, PlayerControlView};
#[path = "engine_library_hand_exchange.rs"]
mod library_hand_exchange;
pub use library_hand_exchange::HandDeploymentView;
#[path = "engine_hand_choices.rs"]
mod hand_choices;
pub use hand_choices::{HandChoiceKind, HandChoiceView};
#[path = "engine_self_granted.rs"]
mod self_granted;
pub use self_granted::{SelfGrantedAbilityKind, SelfGrantedAbilityView};
#[path = "engine_entering_copies.rs"]
mod entering_copies;
#[path = "engine_linked_copies.rs"]
mod linked_copies;
#[path = "engine_self_granted_activations.rs"]
mod self_granted_activations;
pub use self_granted_activations::SelfGrantedActivationContext;
#[path = "engine_cast_subjects.rs"]
mod cast_subjects;
pub use cast_subjects::CastCreatureSubjectsView;
#[path = "engine_hand_exile.rs"]
mod hand_exile;
#[path = "engine_mana_announcements.rs"]
mod mana_announcements;
#[path = "engine_turn_facts.rs"]
mod turn_facts;
pub use turn_facts::TurnFactsView;
#[path = "engine_turn_schedule.rs"]
mod turn_schedule;
pub use turn_schedule::{ExtraTurnView, TurnScheduleView};
#[path = "engine_turn_skips.rs"]
mod turn_skips;
pub use mana_announcements::ManaAnnouncementView;
#[path = "engine_cascade.rs"]
mod cascade;
#[path = "engine_miracle.rs"]
mod miracle;
#[path = "engine_storm.rs"]
mod storm;
pub use cascade::CascadeView;
pub use miracle::MiracleChoiceView;
#[path = "engine_paradigm.rs"]
mod paradigm;
pub use paradigm::{ParadigmView, ResolvedSpellNameView};
#[path = "engine_endgame.rs"]
mod endgame;
#[path = "engine_linked_choices.rs"]
mod linked_choices;
#[path = "engine_zone_replacements.rs"]
mod zone_replacements;
pub use linked_choices::{KeepTypesView, LibraryPile, LibraryPilesView, LinkedExileView};
pub use zone_replacements::{ZoneReplacementChoiceView, ZoneReplacementKind, ZoneRevealView};
#[path = "engine_mana_lots.rs"]
mod mana_lots;
pub use mana_lots::{
    ManaCopySource, ManaLotSpendRange, ManaLotView, ManaRestrictionView, ManaSpellBonusView,
};
#[path = "engine_card_copies.rs"]
mod card_copies;
#[path = "engine_copies.rs"]
mod copies;
#[path = "engine_prepared.rs"]
mod prepared;
#[path = "engine_stack_copies.rs"]
mod stack_copies;
use copies::CopyFaces;
pub use copies::CopyRecipe;
#[path = "engine_copy_grant_origins.rs"]
mod copy_grant_origins;
#[path = "engine_temporary_copy.rs"]
mod temporary_copy;
pub use prepared::PreparationView;
pub use temporary_copy::{
    CopyDeadline, CopyLayerView, CopyOverrides, CopyTypes, TemporaryCopyView,
};
#[path = "engine_ability_target_events.rs"]
mod ability_target_events;
#[path = "engine_copy_batches.rs"]
mod copy_batches;
#[path = "engine_copy_characteristics.rs"]
mod copy_characteristics;
#[path = "engine_return_events.rs"]
mod return_events;
pub use crate::cards::CopyExceptions;
#[path = "engine_target_allocations.rs"]
mod target_allocations;
pub use stack_copies::{CopyEvent, RetargetView};
pub use target_allocations::{TargetAllocation, TargetAmountChoiceView};
const STARTING_LIFE: i128 = 20;
use crate::capacity::{CapacityExt, CapacitySignal};
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
#[path = "engine_milled_selection.rs"]
mod milled_selection;
pub use milled_selection::MilledSelectionView;
#[path = "engine_adventure.rs"]
mod adventure;
#[path = "engine_cast_origins.rs"]
mod cast_origins;
use crate::PermanentType;
pub use cast_origins::{GraveyardTypeUse, ResolutionFreeCasts};
#[path = "engine_cast_permissions.rs"]
mod cast_permissions;
#[path = "engine_turn_cast_grants.rs"]
mod turn_cast_grants;
pub use turn_cast_grants::PlayerCastGrantsView;
#[path = "engine_cast_timing.rs"]
mod cast_timing;
pub use crate::cards::GraveyardCastKind;
pub use cast_permissions::GraveyardCastPermissionView;
#[path = "engine_crew.rs"]
mod crew;
#[path = "engine_saddle.rs"]
mod saddle;
pub use adventure::{AdventurePermission, SpellCharacteristicsView};
pub use crew::{CrewChoiceView, CrewContext, CrewHistory};
pub use saddle::SaddleChoiceView;
pub use teamwork::TeamworkChoiceView;
#[path = "engine_gift.rs"]
mod gift;
#[path = "engine_teamwork.rs"]
mod teamwork;
pub use gift::{GiftContext, GiftDeclaration};
/// Public observations, including Plot/Warp permissions and captured subtype grants.
#[path = "observation_schema.rs"]
mod observation_schema;
pub const OBSERVATION_SCHEMA: u32 = observation_schema::VERSION;
#[path = "engine_cast_payments.rs"]
mod cast_payments;
#[path = "engine_mobilize.rs"]
mod mobilize;
#[path = "engine_plot.rs"]
mod plot;
#[path = "engine_tiered.rs"]
mod tiered;
#[path = "engine_warp.rs"]
mod warp;
pub use cast_payments::{CastPaymentEvent, ManaPaymentDomain};
pub use mobilize::{DelayedSacrificeView, MobilizeChoiceView};
pub use plot::{PlotPaymentView, PlottedCard};
pub use return_links::{DelayedReturnView, ReturnLinkView};
pub use warp::{DelayedWarpExileView, WarpedCard};

#[path = "engine_characteristic_layers.rs"]
mod characteristic_layers;
pub use characteristic_layers::AnimationView;

#[path = "engine_subtypes.rs"]
mod subtypes;
pub use subtypes::GrantedSubtypes;

#[path = "engine_replacements.rs"]
mod replacements;
#[path = "engine_sagas.rs"]
mod sagas;
#[path = "engine_shield_harness.rs"]
mod shield_harness;
pub use replacements::{
    CreatedTokenChoiceView, DamageEventOrderView, DamagePreventionKind, DamagePreventionView,
    ReplacementChoiceView, ReplacementEffectView, TokenGroupView,
};

use crate::shared_vec::SharedVec;
#[path = "engine_additional_costs.rs"]
mod additional_costs;
#[path = "engine_bargain.rs"]
mod bargain;
#[path = "engine_cast_costs.rs"]
mod cast_costs;
#[path = "engine_casualty.rs"]
mod casualty;
pub use cast_costs::{CastCostClause, CastCostRow, CastCostTransaction};
pub use casualty::{CasualtyDeclaration, CasualtyGrant};
#[path = "engine_cast_mana_recording.rs"]
mod cast_mana_recording;
#[path = "engine_cast_rollback_mana.rs"]
mod cast_rollback_mana;
pub use cast_rollback_mana::ManaRollbackView;
#[path = "engine_evidence.rs"]
mod evidence;
pub use evidence::{EvidenceEventView, EvidenceView};
#[path = "engine_library_selection.rs"]
mod library_selection;
pub use library_selection::{EntryTransformChoiceView, LibrarySelectionView};
#[path = "engine_return_attachment.rs"]
mod return_attachment;
pub use return_attachment::EquipmentAttachmentView;

#[path = "engine_forage.rs"]
mod forage;
#[path = "engine_mana_value_parity.rs"]
mod mana_value_parity;
#[path = "engine_target_library.rs"]
mod target_library;
pub use forage::{ForageRoute, ForageView};
pub use mana_value_parity::ManaValueParity;
pub use target_library::{TargetLibraryMovement, TargetLibraryView};
#[path = "engine_combat_allocations.rs"]
mod combat_allocations;
pub use combat_allocations::CombatDamagePlayerView;
#[path = "engine_scalar_x.rs"]
mod scalar_x;
#[path = "engine_spell_costs.rs"]
mod spell_costs;
#[path = "engine_target_costs.rs"]
mod target_costs;
pub use additional_costs::AdditionalCostView;
#[path = "engine_activation_target_costs.rs"]
mod activation_target_costs;
#[path = "engine_activations.rs"]
mod activations;
#[path = "engine_power_up.rs"]
mod power_up;
pub use power_up::PowerUpUseView;
#[path = "engine_connive.rs"]
mod connive;
#[path = "engine_convoke.rs"]
mod convoke;
#[path = "engine_improvise.rs"]
mod improvise;
pub use improvise::ImproviseChoiceView;
#[path = "engine_damage_triggers.rs"]
mod damage_triggers;
#[path = "engine_discard_events.rs"]
mod discard_events;
#[path = "engine_draw_events.rs"]
mod draw_events;
#[path = "engine_endure.rs"]
mod endure;
#[path = "engine_funding.rs"]
mod funding;
#[path = "engine_keyword_counters.rs"]
mod keyword_counters;
#[path = "engine_live_counters.rs"]
mod live_counters;
pub use convoke::{ConvokeChoiceView, ConvokeContribution, ConvokePaymentView};
#[path = "engine_resolution_mana.rs"]
mod resolution_mana;
use funding::FundingGoal;
#[path = "engine_battlefield_groups.rs"]
mod battlefield_groups;
#[path = "engine_blocking.rs"]
mod blocking;
#[path = "engine_combat_restrictions.rs"]
mod combat_restrictions;
#[path = "engine_effect_choices.rs"]
mod effect_choices;
#[path = "engine_exile.rs"]
mod exile;
#[path = "engine_target_exile_play.rs"]
mod target_exile_play;
pub use exile::{ExilePlayExpiry, ExilePlayPermission};
#[path = "engine_explore.rs"]
mod explore;
#[path = "engine_fight.rs"]
mod fight;
#[path = "engine_milling.rs"]
mod milling;
#[path = "engine_return_links.rs"]
mod return_links;
#[path = "engine_roles.rs"]
mod roles;
#[path = "engine_sba.rs"]
mod sba;
pub use roles::{RoleGroupView, RoleTimestampView};
#[path = "engine_faces.rs"]
mod faces;
#[path = "engine_modal_history.rs"]
mod modal_history;
#[cfg(test)]
#[path = "engine_msh_declaration_tests.rs"]
mod msh_declaration_tests;
#[cfg(test)]
#[path = "engine_msh_existing_seventh_tests.rs"]
mod msh_existing_seventh_tests;
#[cfg(test)]
#[path = "engine_msh_faces_tests.rs"]
mod msh_faces_tests;
#[cfg(test)]
#[path = "engine_msh_jennifer_tests.rs"]
mod msh_jennifer_tests;
#[cfg(test)]
#[path = "engine_msh_king_tests.rs"]
mod msh_king_tests;
#[cfg(test)]
#[path = "engine_super_intelligence_tests.rs"]
mod super_intelligence_tests;
pub use faces::FaceView;
pub use modal_history::ModalHistoryView;
#[path = "engine_ability_evaluation.rs"]
mod ability_evaluation;
#[path = "engine_characteristic_query.rs"]
mod characteristic_query;
#[path = "engine_growth_cost.rs"]
mod growth_cost;
#[path = "engine_source_counter_cost.rs"]
mod source_counter_cost;
#[path = "engine_untap_rules.rs"]
mod untap_rules;
pub use ability_evaluation::{AbilityAvailabilityView, GrantedAbilityView};
#[path = "engine_ability_order.rs"]
mod ability_order;
#[path = "engine_ability_timeline.rs"]
mod ability_timeline;
#[path = "engine_return_ability_modification.rs"]
mod return_ability_modification;
#[path = "engine_source_linked_effects.rs"]
mod source_linked_effects;
pub use return_ability_modification::ResolvedAbilityModificationView;
#[cfg(test)]
#[path = "engine_msh_elektra_hellcat_tests.rs"]
mod msh_elektra_hellcat_tests;
#[path = "engine_sneak.rs"]
mod sneak;
pub use source_linked_effects::{SourceLinkedEffectKind, SourceLinkedEffectView};
#[path = "engine_ability_world.rs"]
mod ability_world;
pub use ability_timeline::{AbilitySourceTimestampView, KeywordEffectOrigin, KeywordEffectView};
#[path = "engine_base_power.rs"]
mod base_power;
#[path = "engine_base_stats.rs"]
mod base_stats;
pub use base_stats::{BaseStatsEffectView, ResolvedStatSubjectView};
#[path = "engine_live_static_stats.rs"]
mod live_static_stats;
pub use live_static_stats::LiveStaticSourceView;
#[path = "engine_search.rs"]
mod search;
pub use explore::{ExploreChoiceView, ExploreEventView, ExplorerView, RevealedLibraryCard};
#[path = "engine_sos_tokens.rs"]
mod sos_tokens;
#[path = "engine_stat_values.rs"]
mod stat_values;
#[path = "engine_stats.rs"]
mod stats;
#[path = "engine_steps.rs"]
mod steps;

#[path = "engine_tokens.rs"]
mod tokens;
use crate::rng::Rng;
use crate::{Card, Keywords, Mana, TokenInfo};
use serde::{Deserialize, Serialize};

/// Stable physical-card slot; ceased token slots may be reused.
/// Use `ObjectRef` for a particular game-object incarnation.
pub type CardId = u32;

/// Rules identity within a game branch. Every zone change creates a new incarnation.
/// References are scoped to their game/snapshot; revision guards still protect actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ObjectRef {
    pub card: CardId,
    pub generation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Player {
    P0,
    P1,
}

impl Player {
    pub fn other(self) -> Self {
        match self {
            Self::P0 => Self::P1,
            Self::P1 => Self::P0,
        }
    }
    pub fn index(self) -> usize {
        self as usize
    }
}

#[path = "engine_ability_resolutions.rs"]
mod ability_resolutions;
#[path = "engine_big_core.rs"]
mod big_core;
#[path = "engine_borrowed_activations.rs"]
mod borrowed_activations;
pub use borrowed_activations::BorrowedAbilityView;

pub use ability_resolutions::AbilityResolutionView;
#[path = "engine_programs.rs"]
mod programs;
use programs::Resolution;
#[path = "engine_targeting.rs"]
mod targeting;
use targeting::Announcement;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Zone {
    Library,
    Hand,
    Battlefield,
    Stack,
    Graveyard,
    Exile,
    Command,
    /// Vacant slot after a token ceases to exist; never a game zone.
    Outside,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Step {
    Pregame,
    Upkeep,
    Draw,
    Main1,
    BeginCombat,
    Attackers,
    Blockers,
    FirstStrikeDamage,
    CombatDamage,
    EndCombat,
    Main2,
    End,
    Cleanup,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Target {
    Player(Player),
    Card(CardId),
    Spell(CardId),
    /// Exact identity of a spell copy or ability on the stack.
    Stack(u64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct BoundTarget {
    pub group: u16,
    pub target: Target,
    pub object: Option<ObjectRef>,
}

// Internally tagged Serde enums buffer through Content, which does not expose
// deserialize_i128. Number's visitor preserves both small and large JSON integers.
fn deserialize_damage_amount<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<i128, D::Error> {
    let number = serde_json::Number::deserialize(deserializer)?;
    number.as_i128().ok_or_else(|| {
        serde::de::Error::custom("damage amount is outside the signed 128-bit integer range")
    })
}

/// Choices are incremental for declarations and multi-blocker damage; no combinatorial
/// enumeration of attacker subsets, blocking configurations, or damage partitions.
// Keep one variant/field list for the public enum and its JSON wire decoder.
// Serde's internally tagged Content buffer lacks a u128/i128 visitor. Replaying
// the exact JSON number tokens through the wire decoder also supports from_value
// and nested protocol requests without narrowing large integers.
macro_rules! define_actions {
    ($($variant:ident $( { $($(#[$attribute:meta])* $field:ident: $ty:ty),* $(,)? } )?),* $(,)?) => {
        #[derive(Clone, Debug, PartialEq, Eq, Serialize)]
        #[serde(tag="kind")]
        pub enum Action { $($variant $( { $($(#[$attribute])* $field: $ty),* } )?),* }
        impl<'de> Deserialize<'de> for Action {
            fn deserialize<D:serde::Deserializer<'de>>(deserializer:D)->Result<Self,D::Error>{
                #[derive(Deserialize)]
                #[serde(tag="kind")]
                enum Wire { $($variant $( { $($(#[$attribute])* $field: $ty),* } )?),* }
                let value=serde_json::Value::deserialize(deserializer)?;
                let wide=value.get("amount").and_then(serde_json::Value::as_number)
                    .is_some_and(|n|n.as_i64().is_none()&&n.as_u64().is_none());
                let wire:Wire=if wide {
                    let text=serde_json::to_string(&value).map_err(serde::de::Error::custom)?;
                    serde_json::from_str(&text).map_err(serde::de::Error::custom)?
                } else {serde_json::from_value(value).map_err(serde::de::Error::custom)?};
                Ok(match wire { $(Wire::$variant $( { $($field),* } )? => Self::$variant $( { $($field),* } )?),* })
            }
        }
    }
}
define_actions! {
    AssignLibraryPile { object: ObjectRef, pile: LibraryPile },
    ChooseLibraryPile { pile: LibraryPile },
    KeepPermanentType { object: ObjectRef },
    ChooseZoneReplacement { effect: ZoneReplacementKind },
    FinishStackTargets,
    AllocateTargetAmount { amount: u16 },
    RestoreStackTarget { index: u32 },
    SetStackTarget { index: u32, target: Target },
    ChooseCollectEvidence { collect: bool },
    ChooseEvidence { object: ObjectRef },
    FinishEvidence,
    DeclineEvidence,
    ChooseReplacement { source: ObjectRef, ability: u16 },
    ChooseDamagePrevention { source: ObjectRef, prevention: DamagePreventionKind },
    ChooseDamageEvent { index: u32 },
    ChooseCreatedToken { object: ObjectRef },
    ChooseEndure { put_counters: bool },
    BeginConvoke,
    BeginImprovise,
    ChooseImproviser { object: ObjectRef },
    ChooseConvoker { object: ObjectRef, contribution: ConvokeContribution },
    ChooseMobilizeDefender { defender: CombatDefender },
    ChooseToken { token: Card },
    PutHandCreature { object: ObjectRef },
    PutHandPermanent { object: ObjectRef },
    PutHandLand { object: ObjectRef },
    SelectLand { object: ObjectRef },
    FinishLandSelection,
    DeclineHandEntry,
    ChooseHandDefender { defender: CombatDefender },
    ChooseTeamwork { pay: bool },
    ChooseTeammate { object: ObjectRef },
    FinishTeamwork,
    ChooseSaddler { object: ObjectRef },
    ChooseCrewer { object: ObjectRef },
    FinishCrew,
    FinishSaddle,
    ChooseLibraryDestination { object: ObjectRef, destination: LibraryDestination },
    ChooseManaValueParity { parity: ManaValueParity },
    ChooseForageRoute { route: ForageRoute },
    ChooseForage { object: ObjectRef },
    DeclineForage,
    RevealMiracle,
    DeclineMiracle,
    AnnounceCounterGraveyard { card: CardId },
    ChooseCreatureCounterCost { object: ObjectRef, counter: CounterKind },
    AnnounceAttachedAbility { grantor: ObjectRef, ability: u16 },
    AnnounceSelfGrantedAbility { source: ObjectRef, grant_timestamp: u64 },
    ChooseAdditionalCost {
        object: ObjectRef,
    },
    ChooseExplore { object: ObjectRef, graveyard: bool },
    ChooseCasualty { pay: bool },
    ChooseCastCost {clause:u16},
    PayCasualty {object:ObjectRef},
    ChooseBargain {
        sacrifice: Option<ObjectRef>,
    },
    ChooseX { value: u32 },
    ChooseSourceCounterAmount { amount: u64 },
    ChooseKicker {
        pay: bool,
    },
    ChooseGift {
        beneficiary: Option<Player>,
    },
    ChooseOffspring {
        pay: bool,
    },
    ChooseSpellCost {
        cost: u16,
    },
    ChooseActivationCost {
        cost: u16,
    },
    ChooseConniver {
        object: ObjectRef,
    },
    PayResolutionMana {
        payment: Mana,
    },
    DeclineResolutionMana,
    // Required private reveal selection, revealed discard, or optional exile.
    ChooseHandCard { object: Option<ObjectRef> },
    ChooseRevealedDiscard { object: ObjectRef },
    RecoverMilled {
        object: Option<ObjectRef>,
    },
    SelectMilled { object: ObjectRef },
    FinishMilledSelection,
    ChooseEntryTimestamp { object: ObjectRef },
    ChooseEntryReplacement { source: ObjectRef, intrinsic: bool },
    ChooseEntryValue { value: EntryChoiceValue },
    ChooseEntryLife {
        pay: bool,
    },
    ChooseAuraAttachment {
        object: ObjectRef,
    },
    Keep,
    BeginOnBattlefield { object: ObjectRef },
    FinishOpeningHand,
    CastExiledDuringResolution { object: ObjectRef, face: CastFace },
    FinishResolutionCasts,
    FinishGraveyardSymbolCost,
    Mulligan,
    Bottom {
        card: CardId,
    },
    Pass,
    PlayLand {
        card: CardId,
    },
    TapMana {
        card: CardId,
    },
    // Mana ability granted to this permanent by an exact attached source.
    TapGrantedMana { card: CardId, grantor: ObjectRef, production: Mana },
    AnnounceCast {
        card: CardId,
    },
    PlayGraveyardPermanent { card: CardId, permission: ObjectRef, permanent_type: PermanentType },
    AnnounceFree { card: CardId, face: CastFace, flash_surcharge: bool },
    AnnounceFreeCounterGraveyard { card: CardId, face: CastFace, flash_surcharge: bool },
    AnnounceFreeGraveyardPermanent { card: CardId, face: CastFace, flash_surcharge: bool, permission: ObjectRef, permanent_type: PermanentType },
    AnnounceFreeFromPermission { card: CardId, face: CastFace, flash_surcharge: bool, permission: ObjectRef },
    AnnounceGrantedFlashback { card: CardId, flash_surcharge: bool },
    PayResolutionLife,
    DeclineResolutionLife,
    AnnounceExile {
        card: CardId,
    },
    AnnounceFlashSurcharge { card: CardId },
    AnnounceWarp { card: CardId },
    AnnounceSneak { card: CardId,
        #[serde(skip_serializing_if = "Option::is_none")]
        permission: Option<(ObjectRef, PermanentType)> },
    AnnounceAdventure { card: CardId },
    AnnounceOmen { card: CardId },
    AnnounceBack { card: CardId },
    AnnouncePlot { card: CardId },
    PayPlot { payment: Mana },
    AnnouncePlotted { card: CardId },
    AnnounceFlashback {
        card: CardId,
    },
    AnnounceBorrowedAbility { card: CardId, exiled: ObjectRef, ability: u16, counter_payment: Counters },
    AnnounceBorrowedMana { card: CardId, exiled: ObjectRef, ability: u16, counter_payment: Counters, production: Mana },
    ActivateBorrowedMana { card: CardId, exiled: ObjectRef, ability: u16, counter_payment: Counters, payment: Mana, production: Mana },
    AnnounceActivate {
        card: CardId,
        ability: u16,
        counter_payment: Counters,
    },
    AnnounceMana {
        card: CardId,
        ability: u16,
        counter_payment: Counters,
        production: Mana,
    },
    PayCosts {
        payment: Mana,
    },
    ChooseManaRollback { keep: bool },
    Cast {
        card: CardId,
        target: Option<Target>,
        payment: Mana,
    },
    BeginCast {
        card: CardId,
        payment: Mana,
    },
    Activate {
        card: CardId,
        ability: u16,
        target: Option<Target>,
        payment: Mana,
        counter_payment: Counters,
    },
    ActivateMana {
        card: CardId,
        ability: u16,
        payment: Mana,
        counter_payment: Counters,
        production: Mana,
    },
    ChooseManaLotSpend { amount: u32 },
    ChooseMana {
        production: Mana,
    },
    BeginActivate {
        card: CardId,
        ability: u16,
        payment: Mana,
        counter_payment: Counters,
    },
    ChooseMode {
        mode: u16,
    },
    ChooseTarget {
        target: Target,
    },
    FinishTargets,
    Attack {
        card: CardId,
    },
    // Declare an attack on an exact opposing planeswalker incarnation.
    AttackPlaneswalker {
        card: CardId,
        defender: ObjectRef,
    },
    FinishAttackers,
    Block {
        blocker: CardId,
        attacker: CardId,
    },
    FinishBlockers,
    AssignDamage {
        target: Target,
        #[serde(deserialize_with = "deserialize_damage_amount")]
        amount: i128,
    },
    Discard {
        card: CardId,
    },
    PutTrigger {
        id: u64,
    },
    ChooseSearch {
        card: CardId,
    },
    ChooseSearchZones { library:bool, graveyard:bool },
    FinishSearch,
    ChooseSearchBattlefield { card: CardId },
    ChooseSearchDestination { destination: crate::SearchDestination },
    SelectLibrary { card: CardId },
    FinishLibrarySelection,
    ChooseEquipmentAttachment { equipment: Option<ObjectRef> },
    OrderLibraryBottom { card: CardId },
    ArrangeLibrary {
        card: CardId,
        destination: LibraryDestination,
    },
    ChooseOptional {
        accept: bool,
    },
    Sacrifice {
        object: ObjectRef,
    },
    DeclineEffectPayment,
    KeepRole { object: ObjectRef },
    KeepLegend {
        object: ObjectRef,
    },
    Concede,
}

/// Within each destination, selected first is highest in that ordered group.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LibraryDestination {
    Top,
    Bottom,
    Graveyard,
    Shuffle,
    SecondFromTop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Outcome {
    Winner(Player),
    Draw,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    CapacityExceeded(crate::CapacityExceeded),
    IllegalAction,
    WrongActor,
    GameOver,
    InvalidDeck(String),
    StaleRevision,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CapacityExceeded(error) => write!(f, "CapacityExceeded: {}", error.resource),
            _ => write!(f, "{self:?}"),
        }
    }
}
impl std::error::Error for Error {}

/// Freeform is deliberately explicit: useful for training, not tournament deck legality.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DeckRules {
    #[default]
    Constructed,
    Freeform,
    /// Standard pool frozen at the compiled catalogue version, not host time.
    Standard,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub decks: [Vec<Card>; 2],
    pub seed: u64,
    #[serde(default = "first_player")]
    pub starting_player: Player,
    #[serde(default)]
    pub deck_rules: DeckRules,
}
fn first_player() -> Player {
    Player::P0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
enum CopyKind {
    None,
    Token,
    Spell,
    Card,
    /// Physical card with an effective definition that may itself be synthetic.
    Physical,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Object {
    card: Card,
    copy_kind: CopyKind,
    owner: Player,
    controller: Player,
    zone: Zone,
    generation: u64,
    tapped: bool,
    // True until this creature's controller begins their next turn.
    summoning_sick: bool,
    // Exact current stack incarnation was cast using flashback (CR702.34a).
    flashback: bool,
    damage: i128,
    damaged_this_turn: bool,
    dealt_damage_this_turn: bool,
    deathtouch_since_sba: bool,
    growth: Stats,
    gained_keywords: Keywords,
    counters: InitialCounters,
    used_abilities: u64,
    attached_to: Option<ObjectRef>,
}

impl Object {
    fn is_copy_token(&self) -> bool {
        self.copy_kind == CopyKind::Token
    }
    fn is_spell_copy(&self) -> bool {
        matches!(self.copy_kind, CopyKind::Spell | CopyKind::Card)
    }
    fn is_token(&self) -> bool {
        self.copy_kind != CopyKind::Physical && (self.is_copy_token() || self.card.is_token())
    }
    fn untap(&mut self) {
        if self.tapped {
            if self.counters.stun > 0 {
                self.counters.stun -= 1;
            } else {
                self.tapped = false;
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PlayerState {
    turn_facts: TurnFactsView,
    hexproof_until_end_turn: bool,
    /// Presence of a lasting grant; its authoritative order is in HandSizeState.
    no_maximum_hand_size: bool,
    extra_land_plays_remaining: u64,
    extra_land_plays_granted: u64,
    nonland_permanents_entered_this_turn: u64,
    spells_cast_this_turn: u64,
    instant_sorcery_cast_this_turn: u64,
    cards_drawn_this_turn: u64,
    cards_discarded_this_turn: u64,
    card_left_graveyard_this_turn: bool,
    controlled_creature_died_this_turn: bool,
    life_gain_events_this_turn: u64,
    life_gained_this_turn: u128,
    life_lost_this_turn: u128,
    attacked_this_turn: bool,
    life: i128,
    poison_counters: u64,
    mana: Mana,
    library: SharedVec<CardId>, // top is last, bottom is first
    mulligans: u8,
    kept: bool,
    attempted_empty_draw: bool,
    // Subset of mana[3], retained across steps until this combat ends.
    combat_red_mana: u32,
}

/// How an actual spell was cast; abilities have no cast information.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum CastMethod {
    Normal,
    GraveyardCounterCost,
    Free,
    GrantedFlashback,
    Flashback,
    Miracle,
    Plot,
    Warp,
    Sneak,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CastFace {
    Normal,
    Adventure,
    Omen,
    Back,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CastInfo {
    /// Captured permission to spend mana of any type on this particular spell.
    pub spend_any_mana: bool,
    /// Exact spell replacement: only a stack-to-graveyard move becomes exile.
    pub exile_on_graveyard: bool,
    /// Player who cast this exact spell; stack control changes do not alter history.
    pub caster: Player,
    /// A non-targeted promise, declared before targets; absent until chosen.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gift: Option<GiftDeclaration>,
    /// Chosen characteristics, independent of payment/permission method.
    pub face: CastFace,
    /// Canonical physical identity, distinct from the chosen inset definition.
    pub physical_card: Option<Card>,
    /// Actual mana spent on this cast, including additional mana costs.
    pub payment: Mana,
    pub origin: Zone,
    /// Counter qualification captured before the exile-to-stack transition.
    pub stash_origin: bool,
    pub method: CastMethod,
    /// Defender captured before the non-targeted Sneak return is paid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sneak_defender: Option<CombatDefender>,
    /// Optional additional mana paid to obtain this spell's flash timing.
    pub flash_surcharge: bool,
    /// Explicit optional additional cost, declared before targets; None until chosen.
    pub offspring: Option<bool>,
    /// One fixed optional surcharge, independent of casting method and offspring.
    pub kicker: Option<bool>,
    /// Whether this cast declared the optional teamwork cost. Copied with the spell.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub teamwork: Option<bool>,
    /// Exact cast declaration; its reserved sacrifice is held by the announcement.
    pub bargain: Option<bool>,
    pub evidence: Option<bool>,
}

/// Frozen recipient and immutable program provenance, independent of later attachments.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct AttachmentTriggerView {
    pub grantor: ObjectRef,
    pub granting_card: Card,
    pub recipient: ObjectRef,
    /// Present only for an attack-declaration event.
    pub defender: Option<Player>,
    /// True when the actual creature owns an ability granted by the Equipment.
    pub granted: bool,
    /// Copied static grantor or copy-created grant effect; never the recipient's
    /// independent layer-one acquisition. Frozen with this granted packet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<u64>,
}

/// Exact public source movement observed by a zone-change trigger (CR400.7e).
/// The packet's original `object` and source facts remain its last-known source.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TriggerSourceMoveView {
    pub before: ObjectRef,
    pub after: ObjectRef,
    pub origin: Zone,
    pub destination: Zone,
}

/// A successful action's non-targeted subject, separate from source LKI.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReflexiveSubjectView {
    pub object: ObjectRef,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SpellView {
    /// Declared nonmana counter cost; paid is set only by the committed cost executor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_counter_payment: Option<SourceCounterPaymentView>,
    /// Successful exact exile results paid by this ability's printed-symbol cost.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(
        clippy::box_collection,
        reason = "Rare owned cost results use one pointer in ordinary packets"
    )]
    pub exiled_symbol_cards: Option<Box<Vec<ObjectRef>>>,
    /// Allocated only after a targeted permanent departs; these are public facts.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(clippy::box_collection)]
    pub target_controller_lki: Option<Box<Vec<(ObjectRef, Player)>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gift: Option<Box<GiftContext>>,
    pub reflexive_subject: Option<Box<ReflexiveSubjectView>>,
    pub source_move: Option<Box<TriggerSourceMoveView>>,
    pub borrowed_slot: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_granted_activation: Option<Box<SelfGrantedActivationContext>>,
    pub crew: Option<Box<CrewContext>>,
    pub cast_event: Option<Box<CastPaymentEvent>>,
    /// Exact non-target subjects fixed when the triggering spell was cast.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cast_creature_subjects: Option<Box<CastCreatureSubjectsView>>,
    pub copy_event: Option<Box<stack_copies::CopyEvent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_copy_recipe: Option<Box<CopyRecipe>>,
    /// A newly acquired copied ability is independent even for an identical
    /// blueprint. Printed abilities retain None through removal and restoration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_ability_origin: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captured_artifact_copy: Option<Box<CopyRecipe>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(clippy::box_collection)]
    pub target_allocations: Option<Box<Vec<TargetAllocation>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(clippy::box_collection)]
    pub mana_copy_sources: Option<Box<Vec<(ManaCopySource, u64)>>>,
    pub is_copy: bool,
    pub delayed_sacrifice: Option<Box<DelayedSacrificeView>>,
    pub delayed_warp_exile: Option<Box<DelayedWarpExileView>>,
    pub return_link: Option<Box<ReturnLinkView>>,
    /// Chosen X, retained independently by a spell and its own entry-trigger packets.
    pub x: Option<u32>,
    pub attachment_trigger: Option<Box<AttachmentTriggerView>>,
    pub cast: Option<CastInfo>,
    /// The exact event object; this is a non-targeted reference, never rebound.
    pub entering_object: Option<ObjectRef>,
    /// Historical qualification of an entry trigger; never inherited by reincarnations.
    pub evidence_collected: bool,
    pub id: u64,
    pub kind: StackKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_transform: Option<u64>,
    pub source_card: Card,
    pub source_keywords: Keywords,
    pub source_tapped: bool,
    pub source_attachment: Option<ObjectRef>,
    pub damage_event: Option<DamageTriggerView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub damage_received: Option<Box<DamageReceivedView>>,
    pub creature_combat_event: Option<Box<CreatureCombatEventView>>,
    pub discard_event: Option<DiscardTriggerView>,
    pub draw_event: Option<DrawTriggerView>,
    pub life_event: Option<LifeTriggerView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(clippy::box_collection)]
    pub source_counter_lki: Option<Box<Vec<SourceCounterQuantityView>>>,
    /// Selected public source facts; refreshed before its complete departure batch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_quantity_lki: Option<Box<SourceQuantityView>>,
    /// Exact independent event object and selected public departure facts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_object: Option<Box<EventObjectView>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sacrificed_toughness: Option<Box<i128>>,
    pub source_facts: DamageFacts,
    pub related_stack: Option<StackLink>,
    pub card: CardId,
    pub object: ObjectRef,
    pub targets: Vec<BoundTarget>,
    pub mode: Option<u16>,
    pub controller: Player,
}

impl SpellView {
    fn needs_target_controller_lki(&self) -> bool {
        // Legacy event packets can have no typed ability entry. They cannot
        // contain a typed relational target plan and must not enter program().
        match self.kind {
            StackKind::TriggeredAbility { ability }
                if self
                    .trigger_definition()
                    .triggers()
                    .get(ability as usize)
                    .is_none() =>
            {
                return false;
            }
            StackKind::ActivatedAbility { ability }
                if self
                    .source_card
                    .activations()
                    .get(ability as usize)
                    .is_none() =>
            {
                return false;
            }
            _ => {}
        }
        let needs = |spec: TargetSpec| {
            matches!(
                spec.rule,
                TargetRule::Grouped {
                    constraint: TargetGroupConstraint::DifferentControllers,
                    ..
                }
            )
        };
        self.program().is_some_and(|(plan, _)| match plan {
            TargetPlan::Single(spec) => needs(spec),
            TargetPlan::Groups(groups) => groups.iter().copied().any(needs),
        })
    }
    /// Heap-owned continuation payloads; inline packet storage is counted by its container.
    pub(super) fn owned_bytes(&self) -> usize {
        self.exiled_symbol_cards.as_ref().map_or(0, |r| {
            std::mem::size_of::<Vec<ObjectRef>>() + r.len() * std::mem::size_of::<ObjectRef>()
        }) + self
            .self_granted_activation
            .as_ref()
            .map_or(0, |_| std::mem::size_of::<SelfGrantedActivationContext>())
            + self.cast_creature_subjects.as_ref().map_or(0, |event| {
                std::mem::size_of::<CastCreatureSubjectsView>()
                    + event.objects.len() * std::mem::size_of::<ObjectRef>()
            })
            + self
                .event_object
                .as_ref()
                .map_or(0, |_| std::mem::size_of::<EventObjectView>())
            + self
                .captured_artifact_copy
                .as_ref()
                .map_or(0, |_| std::mem::size_of::<CopyRecipe>())
            + self.mana_copy_sources.as_ref().map_or(0, |sources| {
                std::mem::size_of::<Vec<(ManaCopySource, u64)>>()
                    + sources.len() * std::mem::size_of::<(ManaCopySource, u64)>()
            })
            + self.copy_event.as_ref().map_or(0, |event| {
                std::mem::size_of::<stack_copies::CopyEvent>() + event.spell.owned_bytes()
            })
            + self
                .source_quantity_lki
                .as_ref()
                .map_or(0, |_| std::mem::size_of::<SourceQuantityView>())
            + self.source_counter_lki.as_ref().map_or(0, |facts| {
                std::mem::size_of::<Vec<SourceCounterQuantityView>>()
                    + facts.len() * std::mem::size_of::<SourceCounterQuantityView>()
            })
            + self.target_controller_lki.as_ref().map_or(0, |v| {
                std::mem::size_of::<Vec<(ObjectRef, Player)>>()
                    + v.len() * std::mem::size_of::<(ObjectRef, Player)>()
            })
            + self
                .gift
                .as_ref()
                .map_or(0, |_| std::mem::size_of::<GiftContext>())
            + self
                .source_move
                .as_ref()
                .map_or(0, |_| std::mem::size_of::<TriggerSourceMoveView>())
            + self
                .reflexive_subject
                .as_ref()
                .map_or(0, |_| std::mem::size_of::<ReflexiveSubjectView>())
            + self
                .cast_event
                .as_ref()
                .map_or(0, |_| std::mem::size_of::<CastPaymentEvent>())
            + self
                .source_copy_recipe
                .as_ref()
                .map_or(0, |_| std::mem::size_of::<CopyRecipe>())
            + self
                .return_link
                .as_ref()
                .map_or(0, |_| std::mem::size_of::<ReturnLinkView>())
            + self
                .damage_received
                .as_ref()
                .map_or(0, |_| std::mem::size_of::<DamageReceivedView>())
            + self
                .creature_combat_event
                .as_ref()
                .map_or(0, |_| std::mem::size_of::<CreatureCombatEventView>())
            + self.delayed_sacrifice.as_ref().map_or(0, |d| d.bytes())
            + self.crew.as_ref().map_or(0, |c| c.bytes())
            + self
                .delayed_warp_exile
                .as_ref()
                .map_or(0, |_| std::mem::size_of::<DelayedWarpExileView>())
            + self
                .sacrificed_toughness
                .as_ref()
                .map_or(0, |_| std::mem::size_of::<i128>())
            + self.target_allocations.as_ref().map_or(0, |rows| {
                std::mem::size_of::<Vec<TargetAllocation>>()
                    + rows.len() * std::mem::size_of::<TargetAllocation>()
            })
            + self.targets.len() * std::mem::size_of::<BoundTarget>()
            + self
                .attachment_trigger
                .as_ref()
                .map_or(0, |_| std::mem::size_of::<AttachmentTriggerView>())
    }
    /// Source-relative targets refer to the tracked public incarnation, without
    /// rebinding a later departure/reentry or replacing source LKI.
    pub(crate) fn target_source(&self) -> ObjectRef {
        self.source_move
            .as_ref()
            .map_or(self.object, |event| event.after)
    }

    /// Characteristics of this stack spell, independent of actual mana paid.
    pub fn mana_value(&self) -> u64 {
        self.source_card.fixed_mana_value()
            + if self.kind == StackKind::Spell {
                u64::from(self.source_card.x_symbols()) * u64::from(self.x.unwrap_or(0))
            } else {
                0
            }
    }
    pub(crate) fn trigger_definition(&self) -> Card {
        self.attachment_trigger
            .as_ref()
            .filter(|event| event.granted)
            .map_or(self.source_card, |event| event.granting_card)
    }

    pub(crate) fn modes(&self) -> Option<&'static [SpellMode]> {
        match self.kind {
            StackKind::Spell => match self.source_card.effect() {
                Effect::Modal(modes) => Some(modes),
                _ => None,
            },
            StackKind::TriggeredAbility { ability } => match self.trigger_definition().triggers()
                [ability as usize]
                .program
                .first()
            {
                Some(
                    Instruction::Modal(modes)
                    | Instruction::ModalOnce(modes)
                    | Instruction::ModalOncePerTurn(modes)
                    | Instruction::ModalUpToX { modes, .. },
                ) => Some(modes),
                _ => None,
            },
            StackKind::ActivatedAbility { ability } => {
                match self.source_card.activation(ability).program.first() {
                    Some(Instruction::Modal(modes)) => Some(modes),
                    _ => None,
                }
            }
            StackKind::DelayedSacrifice
            | StackKind::DelayedWarpExile
            | StackKind::CopyEndStepSacrifice
            | StackKind::CasualtyCopy
            | StackKind::DelayedReturn => None,
        }
    }
    pub(crate) fn bounded_mode_count(&self, mode: usize) -> Option<u8> {
        let StackKind::TriggeredAbility { ability } = self.kind else {
            return None;
        };
        match self.trigger_definition().triggers()[ability as usize]
            .program
            .first()
        {
            Some(Instruction::ModalUpToX { counts, .. }) => Some(counts[mode]),
            _ => None,
        }
    }
    pub(crate) fn program(&self) -> Option<(TargetPlan, &'static [Instruction])> {
        if let Some(modes) = self.modes() {
            return self
                .mode
                .and_then(|mode| modes.get(usize::from(mode)))
                .map(|mode| (mode.targets, mode.program));
        }
        match self.kind {
            StackKind::TriggeredAbility { ability } => {
                let a = &self.trigger_definition().triggers()[ability as usize];
                let targets = match a.targets {
                    TargetPlan::Single(mut spec)
                        if matches!(spec.rule, TargetRule::DamagedPlayerPermanent { .. }) =>
                    {
                        let TargetRule::DamagedPlayerPermanent { kinds } = spec.rule else {
                            unreachable!()
                        };
                        let player = self
                            .damage_event
                            .expect("damage-bound target requires event")
                            .player;
                        spec.rule = TargetRule::Object {
                            zone: TargetZone::Battlefield,
                            kinds,
                            relation: if player == self.controller { 0 } else { 1 },
                            other: false,
                            nonland: false,
                            tapped: false,
                            min_mana: 0,
                            max_mana: u32::MAX,
                        };
                        TargetPlan::Single(spec)
                    }
                    TargetPlan::Single(mut spec)
                        if matches!(
                            spec.rule,
                            TargetRule::GraveCreaturePower { maximum: None, .. }
                        ) =>
                    {
                        let TargetRule::GraveCreaturePower {
                            excluded_subtype, ..
                        } = spec.rule
                        else {
                            unreachable!()
                        };
                        spec.rule = TargetRule::GraveCreaturePower {
                            excluded_subtype,
                            maximum: Some(self.source_facts.power),
                        };
                        TargetPlan::Single(spec)
                    }
                    TargetPlan::Single(mut spec) if spec.rule == TargetRule::DefendingCreature => {
                        if let Some(event) = &self.attachment_trigger {
                            spec.rule = TargetRule::Object {
                                zone: TargetZone::Battlefield,
                                kinds: 1,
                                relation: if event.defender == Some(self.controller) {
                                    0
                                } else {
                                    1
                                },
                                other: false,
                                nonland: false,
                                tapped: false,
                                min_mana: 0,
                                max_mana: u32::MAX,
                            };
                        }
                        TargetPlan::Single(spec)
                    }
                    plan => plan,
                };
                Some((targets, a.program))
            }
            StackKind::ActivatedAbility { ability } => {
                let a = &self.source_card.activation(ability);
                Some((a.targets, a.program))
            }
            StackKind::Spell => self
                .source_card
                .spell_program(self.mode)
                .map(|(plan, program)| {
                    let promised = self
                        .cast
                        .is_some_and(|c| matches!(c.gift, Some(GiftDeclaration::Promised { .. })));
                    let plan = if promised {
                        self.source_card
                            .gift()
                            .map_or(plan, |gift| gift.promised_targets)
                    } else {
                        plan
                    };
                    let plan = if self.cast.is_some_and(|c| c.teamwork == Some(true)) {
                        self.source_card
                            .teamwork()
                            .and_then(|s| s.paid_targets)
                            .unwrap_or(plan)
                    } else {
                        plan
                    };
                    let plan = if self.cast.is_some_and(|c| c.kicker == Some(true)) {
                        self.source_card.kicked_targets().unwrap_or(plan)
                    } else {
                        plan
                    };
                    (plan, program)
                }),
            StackKind::CasualtyCopy => Some((
                TargetPlan::NONE,
                &[Instruction::CopyCastSpell {
                    prior_spells: false,
                }],
            )),
            StackKind::CopyEndStepSacrifice => {
                Some((TargetPlan::NONE, &[Instruction::SacrificeSource]))
            }
            StackKind::DelayedReturn => {
                Some((TargetPlan::NONE, &[Instruction::ResolveDelayedReturn]))
            }
            StackKind::DelayedSacrifice => {
                Some((TargetPlan::NONE, &[Instruction::SacrificeDelayedTokens]))
            }
            StackKind::DelayedWarpExile => {
                Some((TargetPlan::NONE, &[Instruction::ExileWarpedPermanent]))
            }
        }
    }
}

/// Exact stack identity and its controller, retained after that item leaves.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct StackLink {
    pub id: u64,
    pub controller: Player,
}

/// Frozen public context for a player-damage trigger. Aggregated 'one or more'
/// events have no single dealer. These references never follow later incarnations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CreatureCombatEventView {
    pub dealer: ObjectRef,
    pub amount: u128,
}

/// Frozen actual damage to one exact creature; its recipient is not a target.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DamageReceivedView {
    pub recipient: ObjectRef,
    pub amount: u128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct DamageTriggerView {
    pub dealer: Option<ObjectRef>,
    pub player: Player,
    pub amount: u128,
    pub combat: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum StackKind {
    Spell,
    TriggeredAbility { ability: u16 },
    ActivatedAbility { ability: u16 },
    DelayedSacrifice,
    DelayedWarpExile,
    CopyEndStepSacrifice,
    CasualtyCopy,
    DelayedReturn,
}

/// The object being attacked; this is not a spell/ability target.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatDefender {
    Player(Player),
    Planeswalker(ObjectRef),
}

/// A selected or committed attack. A departed defender becomes `None` while
/// the creature stays attacking and may still exchange damage with blockers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AttackDeclaration {
    pub card: CardId,
    pub defender: Option<CombatDefender>,
    pub blocked: bool,
}
type Attacker = AttackDeclaration;

#[derive(Clone, Debug, PartialEq, Eq)]
struct DamageChoice {
    source: DamageSource,
    defender: Option<CombatDefender>,
    blockers: Vec<CardId>,
    remaining: i128,
    trample: bool,
    prior_blockers_lethal: bool,
}

/// Damage-relevant characteristics captured while a source exists in its zone.
/// Battlefield status stays true for last-known battlefield sources, not cards
/// that deal damage from their hand or as spells.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct DamageFacts {
    pub token: bool,
    pub controller: Player,
    pub kinds: u16,
    pub colors: u8,
    pub power: i128,
    pub power_above_base: bool,
    pub granted_subtypes: GrantedSubtypes,
    pub card: Card,
    pub battlefield: bool,
    pub counter: bool,
    pub plus_one: bool,
    pub modified: bool,
    pub attacking: bool,
}
impl DamageFacts {
    pub(super) fn is_token(&self) -> bool {
        self.token
    }
    /// Effective colors frozen with the source's other last-known characteristics.
    pub(super) fn colors(&self) -> u8 {
        self.colors
    }
    fn from_view(view: &ObjectView) -> Self {
        Self {
            controller: view.controller,
            kinds: view.kinds,
            colors: view.colors,
            power: view.power,
            power_above_base: view.power_above_base,
            granted_subtypes: view.granted_subtypes,
            card: view.card,
            token: view.is_token(),
            battlefield: view.zone == Zone::Battlefield,
            counter: view.counters.has_any(),
            plus_one: view.counters.plus_one > 0,
            modified: view.modified,
            attacking: view.attacking,
        }
    }
}

/// Source characteristics captured when a damage event is constructed. Damage
/// need not originate from a permanent that survives the simultaneous batch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct DamageSource {
    pub facts: DamageFacts,
    pub object: CardId,
    pub reference: ObjectRef,
    pub controller: Player,
    pub keywords: Keywords,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct DamageEvent {
    /// False for damage explicitly made unpreventable by its event or effect.
    pub preventable: bool,
    pub source: DamageSource,
    pub target: Target,
    pub amount: i128,
    pub combat: bool,
}

/// A public life-gain or life-loss event captured when an observer triggers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct LifeTriggerView {
    pub player: Player,
    pub amount: i128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LifeGainEvent {
    source: DamageSource,
    amount: i128,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Pending {
    SourceCounterAmount {
        player: Player,
    },
    ManaRollback {
        player: Player,
    },
    LinkedChoice {
        player: Player,
    },
    MiracleReveal {
        player: Player,
    },
    ResolutionCasts {
        player: Player,
    },
    ResolutionX {
        player: Player,
    },
    TargetAmount {
        player: Player,
    },
    ZoneReplacement {
        player: Player,
    },
    Retarget {
        player: Player,
    },
    ManaLotSpend {
        player: Player,
    },
    Replacement {
        player: Player,
    },
    CreatedToken {
        player: Player,
    },
    Endure {
        player: Player,
        source: ObjectRef,
        amount: u16,
        token: Card,
    },
    PlotPayment {
        player: Player,
    },
    Improvise {
        player: Player,
    },
    Convoke {
        player: Player,
    },
    EvidenceIntent {
        player: Player,
    },
    Evidence {
        player: Player,
    },
    Mobilize {
        player: Player,
    },
    TeamworkIntent {
        player: Player,
    },
    TeamworkPayment {
        player: Player,
    },
    Saddle {
        player: Player,
    },
    Crew {
        player: Player,
    },
    Explore {
        player: Player,
    },
    TargetLibrary {
        player: Player,
    },
    ManaValueParity {
        player: Player,
    },
    ForageMethod {
        player: Player,
    },
    Forage {
        player: Player,
    },
    Casualty {
        player: Player,
    },
    CastCosts {
        player: Player,
    },
    CasualtyPayment {
        player: Player,
        instance: u16,
    },
    Bargain {
        player: Player,
    },
    X {
        player: Player,
    },
    Kicker {
        player: Player,
    },
    Gift {
        player: Player,
    },
    Offspring {
        player: Player,
    },
    SpellCost {
        player: Player,
    },
    ActivationCost {
        player: Player,
    },
    ConniveOrder {
        player: Player,
    },
    ResolutionMana {
        player: Player,
    },
    ResolutionLife {
        player: Player,
    },
    MilledSelection {
        player: Player,
    },
    HandCardChoice {
        player: Player,
    },
    RevealedDiscard {
        player: Player,
        victim: Player,
        excluded_kinds: u16,
    },
    RecoverMilled {
        player: Player,
        filter: RecoveryFilter,
    },
    EntryTimestamp {
        player: Player,
    },
    EntryReplacement {
        player: Player,
    },
    EntryValue {
        player: Player,
    },
    LandCardName {
        player: Player,
    },
    EntryLife {
        player: Player,
    },
    AuraEntry {
        player: Player,
    },
    OpeningHand {
        player: Player,
        eligible: Box<[Vec<ObjectRef>; 2]>,
    },
    OptionalSearch(Player),
    Mulligan(Player),
    Bottom {
        player: Player,
        remaining: u8,
    },
    Priority,
    Role {
        player: Player,
    },
    Legend {
        player: Player,
    },
    EquipmentAttachment {
        player: Player,
    },
    EffectLibrarySelection {
        player: Player,
    },
    EffectLibrary {
        player: Player,
    },
    EffectSearch {
        player: Player,
    },
    LibraryGraveyardSearch {
        player: Player,
    },
    EffectMana {
        player: Player,
        production: ManaProduction,
    },
    Targets {
        player: Player,
    },
    Modes {
        player: Player,
    },
    AdditionalCost {
        player: Player,
    },
    PayCosts {
        player: Player,
    },
    OrderTriggers {
        player: Player,
    },
    EffectOptional {
        player: Player,
        skip_to: usize,
    },
    TokenChoice {
        player: Player,
        tokens: &'static [Card],
    },
    EffectChoice {
        player: Player,
    },
    HandDefender {
        player: Player,
    },
    EffectDiscard {
        player: Player,
        remaining: usize,
    },
    Attackers,
    Blockers,
    Damage,
    Discard,
    Terminal,
}

/// All mutable rules state, including every suspended choice and future randomness.
/// Card definitions are enum/static data, never cloned object graphs or trait objects.
impl Pending {
    fn owned_bytes(&self) -> usize {
        match self {
            Self::OpeningHand { eligible, .. } => {
                std::mem::size_of::<[Vec<ObjectRef>; 2]>()
                    + eligible
                        .iter()
                        .map(|v| v.len() * std::mem::size_of::<ObjectRef>())
                        .sum::<usize>()
            }
            _ => 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct State {
    library_top: Option<Box<library_top::LibraryTopState>>,
    cast_rollback: Option<Box<cast_rollback::CastRollback>>,
    paradigm: Option<Box<paradigm::ParadigmState>>,
    linked_choices: Option<Box<linked_choices::ChoiceState>>,
    combat_rules: Option<Box<combat_allocations::CombatRulesState>>,
    ending_turn: bool,
    turn_skips: Option<Box<[u64; 2]>>,
    player_control: Option<Box<player_control::PlayerControlState>>,
    turn_schedule: Option<Box<TurnScheduleView>>,
    zone_rules: Option<Box<zone_replacements::ZoneState>>,
    mana_lots: Option<Box<mana_lots::ManaState>>,
    controls: Option<Box<continuous_control::ControlState>>,
    #[allow(clippy::box_collection)]
    temporary_controls: Option<Box<Vec<TemporaryControlView>>>,
    casting_prohibited: bool,
    damage_prevention_prohibited: bool,
    #[allow(clippy::box_collection)]
    temporary_abilities: Option<Box<Vec<TemporaryAbilityView>>>,
    #[allow(clippy::box_collection)]
    self_granted_abilities: Option<Box<Vec<self_granted::Row>>>,
    #[allow(clippy::box_collection)]
    suspected: Option<Box<Vec<ObjectRef>>>,
    #[allow(clippy::box_collection)]
    death_exile_replacements: Option<Box<Vec<ExileDeathReplacementView>>>,
    roles: Option<Box<roles::RoleState>>,
    hand_size: Option<Box<hand_size::HandSizeState>>,
    replacements: Option<Box<replacements::ReplacementState>>,
    // Sparse registry: keep an eight-byte pointer in the common empty state
    // rather than an inline Vec header for games with no delayed obligation.
    #[allow(clippy::box_collection)]
    delayed_sacrifices: Option<Box<Vec<mobilize::DelayedBatch>>>,
    // Cold pointer keeps games without delayed returns allocation-free.
    #[allow(clippy::box_collection)]
    delayed_returns: Option<Box<Vec<return_links::DelayedReturn>>>,
    mobilize: Option<Box<mobilize::MobilizePlan>>,
    saddled: Vec<ObjectRef>,
    saddle_payment: Option<Box<saddle::SaddlePayment>>,
    crew: Option<Box<crew::CrewState>>,
    objects: SharedVec<Object>,
    named_counters: Option<SharedVec<live_counters::NamedCounterRow>>,
    creature_damage_history: Option<SharedVec<CreatureDamageHistoryView>>,
    tap_history: tap_history::Rows,
    counter_placement_history: counter_placement_history::Rows,
    cast_entry_counters: cast_entry_counters::Rows,
    entry_values: entry_choices::Rows,
    entered_this_turn: Option<SharedVec<ObjectRef>>,
    power_up_uses: power_up::Rows,
    copy_recipes: copies::Rows,
    temporary_copies: temporary_copy::Rows,
    copy_grant_origins: copy_grant_origins::Rows,
    graveyard_entries: Option<SharedVec<ObjectRef>>,
    nightmare_cull: Option<Box<copy_characteristics::NightmareCull>>,
    stack_copy_data: Option<Box<stack_copies::CopyState>>,
    #[allow(clippy::box_collection)]
    delayed_artifact_copies: Option<Box<Vec<SpellView>>>,
    #[allow(
        clippy::box_collection,
        reason = "Rare delayed main-phase packets use one pointer in ordinary states"
    )]
    delayed_main_mana: Option<Box<Vec<SpellView>>>,
    modal_history: modal_history::Rows,
    face_changes: faces::Rows,
    ability_resolutions: ability_resolutions::Rows,
    attacked_this_turn: SharedVec<ObjectRef>,
    cards_exiled_this_turn: bool,
    creatures_died_this_turn: u64,
    free_tokens: Vec<CardId>,
    static_sources: Vec<CardId>,
    cast_sources: Vec<CardId>,
    attack_sources: Vec<CardId>,
    tap_sources: Vec<CardId>,
    damage_sources: Vec<CardId>,
    discard_sources: Vec<CardId>,
    draw_sources: Vec<CardId>,
    cleanup_discards: Vec<CardId>,
    triggered_this_turn: Vec<TriggerUse>,
    step_sources: Vec<CardId>,
    life_sources: Vec<CardId>,
    counter_sources: Vec<CardId>,
    death_sources: Vec<CardId>,
    ability_target_sources: ability_target_events::Sources,
    attachment_sources: Vec<CardId>,
    exile_links: Vec<ExileLink>,
    exile_permissions: Vec<ExilePlayPermission>,
    #[allow(
        clippy::box_collection,
        reason = "Cold rule state keeps one pointer in games without this mechanic"
    )]
    hand_play_bans: Option<Box<Vec<Player>>>,
    #[allow(
        clippy::box_collection,
        reason = "Cold rule state keeps one pointer in games without this mechanic"
    )]
    borrowed_links: Option<Box<Vec<borrowed_activations::Link>>>,
    #[allow(
        clippy::box_collection,
        reason = "Cold rule state keeps one pointer in games without this mechanic"
    )]
    borrowed_uses: Option<Box<Vec<borrowed_activations::Use>>>,
    used_free_hand_permissions: Option<SharedVec<ObjectRef>>,
    graveyard_cast_permissions: cast_permissions::Rows,
    turn_cast_grants: Option<Box<[PlayerCastGrantsView; 2]>>,
    graveyard_type_uses: Option<SharedVec<GraveyardTypeUse>>,
    resolution_free_casts: Option<Box<ResolutionFreeCasts>>,
    miracle: Option<Box<miracle::MiracleState>>,
    preparations: prepared::Rows,
    adventures: Option<Box<adventure::AdventureState>>,
    plot: Option<Box<plot::PlotState>>,
    warp: Option<Box<warp::WarpState>>,
    known_hand: Vec<ObjectRef>,
    explore_data: Option<Box<explore::ExploreState>>,
    evidence_data: Option<Box<evidence::EvidenceState>>,
    last_target_library_move: Option<Box<TargetLibraryMovement>>,
    players: [PlayerState; 2],
    rng: Rng,
    active: Player,
    priority: Player,
    starting: Player,
    turn: u32,
    step: Step,
    pending: Pending,
    sba_batch: Option<Box<sba::SbaBatch>>,
    entry_batch: Option<Box<entries::EntryBatch>>,
    mulligan_round: [bool; 2],
    stack: Vec<SpellView>,
    resolving: Option<Resolution>,
    announcing: Option<Announcement>,
    #[allow(
        clippy::box_collection,
        reason = "Cold nested mana frames keep ordinary states sparse"
    )]
    mana_announcement_stack: Option<Box<Vec<mana_announcements::ManaAnnouncement>>>,
    pending_triggers: Vec<SpellView>,
    ordering_triggers: Vec<SpellView>,
    next_stack_id: u64,
    passes: u8,
    land_played: bool,
    continuous_land_plays_used: u64,
    attack_declared: bool,
    attackers: Vec<Attacker>,
    blocking: Vec<CardId>, // Remains blocking even after its attacker leaves combat.
    blocks: Vec<(CardId, CardId)>, // blocker, attacker
    selected_attackers: Vec<Attacker>,
    selected_blocks: Vec<(CardId, CardId)>,
    hits: Vec<DamageEvent>,
    damage_choices: Vec<DamageChoice>,
    first_strikers: Vec<CardId>,
    outcome: Option<Outcome>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot(State);

/// Cloning produces an independent search branch. Restore accepts a typed complete
/// snapshot and advances the external revision, so old action indices stay invalid.
#[derive(Clone, Debug)]
pub struct Game {
    state: State,
    revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EndureChoiceView {
    pub source: ObjectRef,
    pub amount: u16,
    pub token: Card,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ObjectView {
    pub prepared: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub harnessed: Option<bool>,
    /// Frozen effective nonkeyword abilities only while a removal effect exists.
    /// None is the allocation-free complete printed/default-grant case.
    pub abilities: Option<Box<AbilityAvailabilityView>>,
    /// Current card-type bits; Card is the effective immutable rules definition.
    /// A temporary copy retains its physical identity in `temporary_copy`.
    pub kinds: u16,
    pub colors: u8,
    /// Frozen layer-four supertype change, absent when the recipe is sufficient.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_legendary: Option<bool>,
    pub granted_subtypes: GrantedSubtypes,
    pub entry_choice: Option<EntryChoiceValue>,
    pub saddled: bool,
    pub id: CardId,
    pub reference: ObjectRef,
    pub card: Card,
    /// Frozen effective rules name; independent of lookup/serialization identity.
    pub name: &'static str,
    pub owner: Player,
    pub controller: Player,
    pub zone: Zone,
    pub tapped: bool,
    pub summoning_sick: bool,
    pub damage: i128,
    pub damaged_this_turn: bool,
    pub dealt_damage_this_turn: bool,
    pub counters: Counters,
    pub used_abilities: u64,
    /// Exhaust uses persist across turns for this exact permanent incarnation.
    pub exhausted_abilities: u64,
    pub entered_this_turn: bool,
    pub power_up_uses: Vec<PowerUpUseView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub face: Option<faces::FaceView>,
    pub attached_to: Option<ObjectRef>,
    pub modified: bool,
    pub attacking: bool,
    pub power: i128,
    pub toughness: i128,
    /// Frozen effective power exceeds its CR208.4b base, before damage/departure.
    pub power_above_base: bool,
    pub keywords: Keywords,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copy_recipe: Option<CopyRecipe>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temporary_copy: Option<Box<TemporaryCopyView>>,
    /// Identity of a continuously provided ability after this object has copied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provided_ability_origin: Option<u64>,
    pub token: Option<TokenInfo>,
    pub emblem: Option<&'static EmblemInfo>,
}

impl ObjectView {
    pub(super) fn copied_grant_origin(&self) -> Option<u64> {
        self.provided_ability_origin
    }
    pub(super) fn temporary_copy_bytes(&self) -> usize {
        self.temporary_copy.as_ref().map_or(0, |view| {
            std::mem::size_of::<TemporaryCopyView>()
                + view.layers.len() * std::mem::size_of::<CopyLayerView>()
        })
    }
    pub fn is_token(&self) -> bool {
        self.token.is_some()
    }
    pub fn is_legendary(&self) -> bool {
        self.effective_legendary.unwrap_or_else(|| {
            self.copy_recipe
                .and_then(|recipe| recipe.overrides)
                .and_then(|value| value.legendary)
                .unwrap_or_else(|| self.card.is_legendary())
        })
    }
    /// Captured rules name, retained even after this object changes or leaves play.
    pub fn name(&self) -> &'static str {
        self.name
    }
}

#[derive(Default)]
struct ZoneObservers {
    graveyard: Option<std::sync::Arc<[ObjectView]>>,
    returned_hand: Option<std::sync::Arc<[ObjectView]>>,
}

/// A single zone transition with the departing object's last-known characteristics.
/// Engine internals consume this value; observations do not expose hidden transitions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZoneChange {
    pub(super) cast_event: Option<Box<CastPaymentEvent>>,
    /// CR122.6a: intrinsic entry counters are put on by the incoming controller.
    pub(super) entry_counter_player: Option<Player>,
    pub(super) graveyard_observers: Option<std::sync::Arc<[ObjectView]>>,
    pub(super) returned_hand_observers: Option<std::sync::Arc<[ObjectView]>>,
    pub gift: Option<Box<GiftContext>>,
    pub was_cast: bool,
    pub before: ObjectView,
    pub after: ObjectRef,
    pub destination: Zone,
    pub moved: bool,
    /// Explicit cause for an actual sacrifice; never inferred from the destination.
    pub sacrificed_by: Option<Player>,
    /// Historical casting qualification, frozen for this single entry incarnation.
    pub offspring_paid: bool,
    /// Paid qualification belongs only to this exact entry event.
    pub kicked: bool,
    pub bargained: bool,
    pub x: u32,
    pub evidence_collected: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct PlayerView {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_cast_grants: Option<PlayerCastGrantsView>,
    pub hexproof_until_end_turn: bool,
    pub turns_to_skip: u64,
    /// Additional land plays granted until the current turn ends.
    pub extra_land_plays_remaining: u64,
    pub no_maximum_hand_size: bool,
    /// Historical actual entries this turn; departures do not remove them.
    pub nonland_permanents_entered_this_turn: u64,
    pub spells_cast_this_turn: u64,
    pub cards_drawn_this_turn: u64,
    pub cards_discarded_this_turn: u64,
    pub card_left_graveyard_this_turn: bool,
    pub controlled_creature_died_this_turn: bool,
    pub life_gain_events_this_turn: u64,
    pub life_gained_this_turn: u128,
    pub life_lost_this_turn: u128,
    pub attacked_this_turn: bool,
    pub life: i128,
    pub poison_counters: u64,
    pub mana: Mana,
    pub combat_red_mana: u32,
    pub library_count: usize,
    pub hand_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PaymentRequestView {
    pub required: Mana,
    pub generic: u32,
    pub counters: Counters,
    pub production: Option<Mana>,
}

/// The choosing player's uncommitted announcement costs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PendingPaymentView {
    pub mana: Mana,
    pub counters: Counters,
}

/// Legal nonhybrid equivalents of a currently announced hybrid spell cost.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SpellCostChoiceView {
    pub index: u16,
    pub cost: ManaCost,
}

/// Legal modes of the currently announced modal spell.
#[derive(Clone, Debug, Serialize)]
pub struct ModeChoiceView {
    pub mode: u16,
    pub text: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_mana: Option<ManaCost>,
}

/// Private cards and partial order of a resolving scry/surveil choice.
#[derive(Clone, Debug, Serialize)]
pub struct LibraryChoiceView {
    pub surveil: bool,
    pub remaining: Vec<ObjectView>,
    pub top: Vec<ObjectView>,
    pub bottom: Vec<ObjectView>,
    pub graveyard: Vec<ObjectView>,
}

/// Private library contents and still-uncommitted selections of a search.
#[derive(Clone, Debug, Serialize)]
pub struct SearchChoiceView {
    pub filter: crate::SearchFilter,
    pub minimum: u16,
    pub maximum: u16,
    pub destination: crate::SearchDestination,
    pub tapped: bool,
    pub reveal: bool,
    pub cards: Vec<ObjectView>,
    pub selected: Vec<CardId>,
}

/// Publicly revealed selected cards whose destinations are being chosen.
#[derive(Clone, Debug, Serialize)]
pub struct SearchDestinationChoiceView {
    pub player: Player,
    pub cards: Vec<ObjectView>,
    pub destinations: [crate::SearchDestination; 2],
}

/// Revealed search results whose split destinations are being assigned.
#[derive(Clone, Debug, Serialize)]
pub struct SearchDistributionView {
    pub player: Player,
    pub battlefield_count: u16,
    pub candidates: Vec<ObjectView>,
    pub battlefield: Vec<CardId>,
}

/// All milled identities and legal recovery candidates are public graveyard facts.
#[derive(Clone, Debug, Serialize)]
pub struct RecoveryChoiceView {
    pub player: Player,
    pub filter: RecoveryFilter,
    pub milled: Vec<ObjectRef>,
    pub candidates: Vec<ObjectRef>,
}

/// Private discard costs and public sacrifice choices suspended during resolution.
#[derive(Clone, Debug, Serialize)]
pub struct EffectChoiceView {
    pub kind: EffectChoiceKind,
    pub amount: u32,
    pub optional: bool,
    pub required: [u32; 2],
    pub selected: [Vec<ObjectRef>; 2],
    pub candidates: Vec<ObjectRef>,
    pub defender_choice: bool,
}

/// Public state-based choices; earlier APNAP choices stay visible to later players.
#[derive(Clone, Debug, Serialize)]
pub struct LegendGroupView {
    pub controller: Player,
    pub name: &'static str,
    pub objects: Vec<ObjectRef>,
    pub keep: Option<ObjectRef>,
}

/// Public face-up exile relation, bound to both exact incarnations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ExileLink {
    pub source: ObjectRef,
    pub exiled: ObjectRef,
    pub return_zone: Zone,
}

/// A public triggered-ability limit consumed by an exact source incarnation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct TriggerUse {
    pub source: ObjectRef,
    pub ability: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<u64>,
}

/// Every integer in this inclusive range is a legal damage assignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct DamageAssignmentRange {
    pub source: ObjectRef,
    pub target: Target,
    pub minimum: i128,
    pub maximum: i128,
}
/// Every integer in this inclusive interval is legal; intervals are sorted and disjoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct XInterval {
    pub minimum: u32,
    pub maximum: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct XChoiceView {
    pub card: ObjectRef,
    pub intervals: Vec<XInterval>,
}
impl XChoiceView {
    pub fn contains(&self, value: u32) -> bool {
        self.intervals
            .iter()
            .any(|r| value >= r.minimum && value <= r.maximum)
    }
}

/// A declared/paid amount is independent of all later source characteristics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SourceCounterPaymentView {
    pub kind: CounterKind,
    pub amount: u64,
    pub paid: bool,
}
/// Every integer in the inclusive domain is a legal quantity declaration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SourceCounterChoiceView {
    pub source: ObjectRef,
    pub kind: CounterKind,
    pub maximum: u64,
}
/// Complete action space: concrete choices and full numeric payment/damage domains.
#[derive(Clone, Debug, Serialize)]
pub struct LegalActionSpace {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_counter_choice: Option<SourceCounterChoiceView>,
    pub mana_lot_spend: Option<ManaLotSpendRange>,
    pub mana_payment: Option<ManaPaymentDomain>,
    pub choices: Vec<Action>,
    pub x_choice: Option<XChoiceView>,
    pub damage_assignment: Option<DamageAssignmentRange>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Observation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_control: Option<PlayerControlView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub library_cast_look: Option<LibraryCastLookView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zone_search: Option<ZoneSearchView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_counter_choice: Option<SourceCounterChoiceView>,
    /// Frozen relevant public events; present only with a public historical observer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_facts: Option<[TurnFactsView; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_library_reveal: Option<LibraryRevealEvent>,
    pub paradigms: Vec<ParadigmView>,
    pub resolved_spell_names: Vec<ResolvedSpellNameView>,
    pub preparations: Vec<PreparationView>,
    pub casualty_declarations: Vec<CasualtyDeclaration>,
    pub cast_costs: Option<CastCostTransaction>,
    pub mana_rollback: Option<ManaRollbackView>,
    pub library_piles: Option<LibraryPilesView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hand_choice: Option<HandChoiceView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hand_deployment: Option<HandDeploymentView>,
    pub keep_types: Option<KeepTypesView>,
    pub linked_exile: Vec<LinkedExileView>,
    pub mana_lots: Vec<ManaLotView>,
    pub mana_lot_spend: Option<ManaLotSpendRange>,
    pub creature_damage_history: Vec<CreatureDamageHistoryView>,
    pub counter_placement_history: Vec<CounterPlacementHistoryView>,
    pub cast_entry_counters: Vec<CastEntryCounterView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toughness_combat_damage: Option<[bool; 2]>,
    pub additional_combats: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_schedule: Option<TurnScheduleView>,
    pub must_be_blocked: Vec<ObjectRef>,
    pub combat_damaged_players: Vec<combat_allocations::CombatDamagePlayerView>,
    pub animations: Vec<AnimationView>,
    pub temporary_controls: Vec<TemporaryControlView>,
    pub casting_prohibited: bool,
    pub damage_prevention_prohibited: bool,
    pub death_exile_replacements: Vec<ExileDeathReplacementView>,
    pub temporary_abilities: Vec<TemporaryAbilityView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_granted_abilities: Option<Vec<SelfGrantedAbilityView>>,
    pub suspected: Vec<ObjectRef>,
    pub replacement_choice: Option<ReplacementChoiceView>,
    pub zone_replacement_choice: Option<ZoneReplacementChoiceView>,
    pub zone_reveals: Vec<ZoneRevealView>,
    pub created_token_choice: Option<CreatedTokenChoiceView>,
    /// Complete exact numeric cost-payment space, visible only to its actor.
    pub mana_payment: Option<ManaPaymentDomain>,
    pub endure_choice: Option<EndureChoiceView>,
    pub convoke_choice: Option<ConvokeChoiceView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub improvise_choice: Option<ImproviseChoiceView>,
    pub delayed_sacrifices: Vec<DelayedSacrificeView>,
    pub delayed_artifact_copies: Vec<SpellView>,
    pub delayed_main_mana: Vec<SpellView>,
    pub delayed_returns: Vec<DelayedReturnView>,
    /// Exact public exile objects awaiting immediate return after departure events.
    pub immediate_blink_return: Option<Vec<ReturnLinkView>>,
    pub mobilize_choice: Option<MobilizeChoiceView>,
    pub saddle_choice: Option<SaddleChoiceView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub teamwork_choice: Option<TeamworkChoiceView>,
    /// Complete scalar domain, visible only to its choosing actor.
    pub x_choice: Option<XChoiceView>,
    pub schema: u32,
    /// Number of distinct declared attacker objects this turn, including departures.
    pub creatures_attacked_this_turn: usize,
    /// Actual battlefield-to-graveyard creature transitions this turn, including tokens.
    pub cards_exiled_this_turn: bool,
    pub creatures_died_this_turn: u64,
    pub graveyard_entries_this_turn: Vec<ObjectRef>,
    pub catalogue: &'static str,
    pub viewer: Player,
    pub revision: u64,
    pub actor: Option<Player>,
    pub active: Player,
    pub turn: u32,
    pub step: Step,
    pub decision: &'static str,
    pub players: [PlayerView; 2],
    pub objects: Vec<ObjectView>,
    /// Public modes already chosen for exact source incarnations.
    pub modal_history: Vec<ModalHistoryView>,
    pub exile_links: Vec<ExileLink>,
    pub exile_permissions: Vec<ExilePlayPermission>,
    pub borrowed_abilities: Vec<BorrowedAbilityView>,
    pub graveyard_cast_permissions: Vec<GraveyardCastPermissionView>,
    pub graveyard_type_uses: Vec<GraveyardTypeUse>,
    pub resolution_free_casts: Option<ResolutionFreeCasts>,
    pub cascade: Option<CascadeView>,
    pub miracle: Option<MiracleChoiceView>,
    pub adventures: Vec<AdventurePermission>,
    pub spell_characteristics: Vec<SpellCharacteristicsView>,
    pub plotted_cards: Vec<PlottedCard>,
    pub crew_choice: Option<CrewChoiceView>,
    pub crewed_vehicles: Vec<CrewHistory>,
    pub warped_cards: Vec<WarpedCard>,
    pub delayed_warp_exiles: Vec<DelayedWarpExileView>,
    pub plot_payment: Option<PlotPaymentView>,
    pub triggered_this_turn: Vec<TriggerUse>,
    pub ability_resolutions: Vec<AbilityResolutionView>,
    pub role_choices: Vec<RoleGroupView>,
    pub role_timestamps: Vec<RoleTimestampView>,
    pub base_stat_effects: Vec<BaseStatsEffectView>,
    pub live_static_sources: Vec<LiveStaticSourceView>,
    pub resolved_stat_subjects: Vec<ResolvedStatSubjectView>,
    pub ability_source_timestamps: Vec<AbilitySourceTimestampView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hand_size_timeline: Option<HandSizeTimelineView>,
    pub keyword_effects: Vec<KeywordEffectView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_linked_effects: Option<Vec<SourceLinkedEffectView>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_ability_modifications: Option<Vec<ResolvedAbilityModificationView>>,
    pub legend_choices: Vec<LegendGroupView>,
    pub stack: Vec<SpellView>,
    pub resolving: Option<SpellView>,
    pub retarget: Option<RetargetView>,
    pub target_amount: Option<TargetAmountChoiceView>,
    pub delayed_spell_copies: Vec<SpellView>,
    pub group_move_count: Option<u64>,
    pub optional_search_sequence: Option<OptionalSearchSequence>,
    pub announcing: Option<SpellView>,
    pub announcing_free_hand_permission: Option<ObjectRef>,
    pub used_free_hand_permissions: Vec<ObjectRef>,
    pub suspended_mana_announcements: Vec<ManaAnnouncementView>,
    pub announcing_payment: Option<PendingPaymentView>,
    pub mana_choices: Vec<Mana>,
    pub mode_choices: Vec<ModeChoiceView>,
    pub cost_choices: Vec<SpellCostChoiceView>,
    pub announcing_cost: Option<ManaCost>,
    pub targeting_group: Option<u16>,
    pub library_choice: Option<LibraryChoiceView>,
    pub library_selection: Option<LibrarySelectionView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_transform: Option<EntryTransformChoiceView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equipment_attachment: Option<EquipmentAttachmentView>,
    pub target_library: Option<TargetLibraryView>,
    pub last_target_library_move: Option<TargetLibraryMovement>,
    pub recovery_choice: Option<RecoveryChoiceView>,
    pub milled_choice: Option<MilledSelectionView>,
    pub search_choice: Option<SearchChoiceView>,
    pub search_distribution: Option<SearchDistributionView>,
    pub search_destination_choice: Option<SearchDestinationChoiceView>,
    pub effect_choice: Option<EffectChoiceView>,
    pub entry_value: Option<entry_choices::EntryChoiceView>,
    pub aura_entry: Option<entries::AuraEntryView>,
    pub entry_life: Option<entries::EntryLifeView>,
    pub entry_replacement: Option<entry_tap::EntryTapView>,
    pub entry_timestamps: Option<entries::EntryTimestampView>,
    pub additional_cost: Option<AdditionalCostView>,
    pub forage: Option<ForageView>,
    pub evidence: Option<EvidenceView>,
    pub last_evidence: Option<EvidenceEventView>,
    pub connive_choice: Option<connive::ConniveChoiceView>,
    pub known_hand: Vec<ObjectRef>,
    pub explore_choice: Option<ExploreChoiceView>,
    pub last_explore: Option<ExploreEventView>,
    pub known_library_top: [Option<RevealedLibraryCard>; 2],
    pub payment_request: Option<PaymentRequestView>,
    pub resolution_payment: Option<resolution_mana::ResolutionPaymentView>,
    pub waiting_triggers: Vec<SpellView>,
    pub effect_discards: Vec<CardId>,
    pub attackers: Vec<CardId>,
    pub attack_declarations: Vec<AttackDeclaration>,
    pub blocking: Vec<CardId>,
    pub blocks: Vec<(CardId, CardId)>,
    pub damage_assigned: Vec<DamageEvent>,
    pub damage_source: Option<CardId>,
    pub damage_assignment: Option<DamageAssignmentRange>,
    pub outcome: Option<Outcome>,
    pub actions: Vec<Action>,
}

impl Game {
    pub fn new(config: Config) -> Result<Self, Error> {
        for deck in &config.decks {
            // Explicit representation bound, never silent truncation.
            if deck.len() < 7 || deck.len() > 65_535 {
                return Err(Error::InvalidDeck(
                    "supported deck size is 7..=65535".into(),
                ));
            }
            if config.deck_rules != DeckRules::Freeform {
                if deck.len() < 60 {
                    return Err(Error::InvalidDeck("constructed needs 60 cards".into()));
                }
                let mut counts = vec![0u8; Card::catalogue_len()];
                for &card in deck {
                    if let Some(limit) = card.info().deck_limit {
                        counts[card.index()] = counts[card.index()].saturating_add(1);
                        if counts[card.index()] > limit {
                            return Err(Error::InvalidDeck(format!(
                                "more than {limit} {}",
                                card.name()
                            )));
                        }
                    }
                }
            }
            for &card in deck {
                if card.is_token()
                    || card.is_emblem()
                    || card.inset_parent().is_some()
                    || card.face_parent().is_some()
                {
                    return Err(Error::InvalidDeck(
                        if card.face_parent().is_some() {
                            "back face definitions cannot be submitted in decks"
                        } else if card.inset_parent().is_some() {
                            "inset spell definitions cannot be submitted in decks"
                        } else if card.is_emblem() {
                            "emblems cannot be submitted in decks"
                        } else {
                            "tokens cannot be submitted in decks"
                        }
                        .into(),
                    ));
                }
                if config.deck_rules == DeckRules::Standard && !card.is_standard_legal() {
                    return Err(Error::InvalidDeck(format!(
                        "not Standard legal in this catalogue: {}",
                        card.name()
                    )));
                }
                if !card.is_supported() {
                    return Err(Error::InvalidDeck(format!(
                        "rules not implemented: {}",
                        card.name()
                    )));
                }
            }
        }
        let mut objects = Vec::with_capacity(config.decks.iter().map(Vec::len).sum());
        let mut players = std::array::from_fn(|_| PlayerState {
            turn_facts: TurnFactsView::default(),
            hexproof_until_end_turn: false,
            no_maximum_hand_size: false,
            extra_land_plays_remaining: 0,
            extra_land_plays_granted: 0,
            nonland_permanents_entered_this_turn: 0,
            spells_cast_this_turn: 0,
            instant_sorcery_cast_this_turn: 0,
            cards_drawn_this_turn: 0,
            cards_discarded_this_turn: 0,
            card_left_graveyard_this_turn: false,
            controlled_creature_died_this_turn: false,
            life_gain_events_this_turn: 0,
            life_gained_this_turn: 0,
            life_lost_this_turn: 0,
            attacked_this_turn: false,
            life: STARTING_LIFE,
            poison_counters: 0,
            mana: [0; 6],
            library: SharedVec::default(),
            mulligans: 0,
            kept: false,
            attempted_empty_draw: false,
            combat_red_mana: 0,
        });
        let mut rng = Rng(config.seed);
        for (p, deck) in [Player::P0, Player::P1].into_iter().zip(config.decks) {
            for card in deck {
                let id = objects.len() as CardId;
                objects.push(Object {
                    card,
                    copy_kind: CopyKind::None,
                    owner: p,
                    controller: p,
                    zone: Zone::Library,
                    generation: 0,
                    tapped: false,
                    summoning_sick: false,
                    flashback: false,
                    damage: 0,
                    damaged_this_turn: false,
                    dealt_damage_this_turn: false,
                    deathtouch_since_sba: false,
                    growth: Stats::default(),
                    gained_keywords: Keywords::NONE,
                    counters: InitialCounters::default(),
                    used_abilities: 0,
                    attached_to: None,
                });
                players[p.index()].library.push(id);
            }
            rng.shuffle(&mut players[p.index()].library);
        }
        let mut game = Self {
            revision: 0,
            state: State {
                library_top: None,
                cast_rollback: None,
                paradigm: None,
                ending_turn: false,
                turn_skips: None,
                player_control: None,
                turn_schedule: None,
                mana_lots: None,
                linked_choices: None,
                zone_rules: None,
                controls: None,
                temporary_controls: None,
                casting_prohibited: false,
                damage_prevention_prohibited: false,
                replacements: None,
                objects: objects.into(),
                named_counters: None,
                combat_rules: None,
                creature_damage_history: None,
                tap_history: None,
                counter_placement_history: None,
                cast_entry_counters: None,
                entry_values: None,
                entered_this_turn: None,
                power_up_uses: None,
                copy_recipes: None,
                temporary_copies: None,
                copy_grant_origins: None,
                graveyard_entries: None,
                nightmare_cull: None,
                stack_copy_data: None,
                delayed_artifact_copies: None,
                delayed_main_mana: None,
                modal_history: None,
                face_changes: None,
                ability_resolutions: None,
                attacked_this_turn: Vec::new().into(),
                cards_exiled_this_turn: false,
                creatures_died_this_turn: 0,
                free_tokens: Vec::new(),
                static_sources: Vec::new(),
                cast_sources: Vec::new(),
                attack_sources: Vec::new(),
                tap_sources: Vec::new(),
                damage_sources: Vec::new(),
                discard_sources: Vec::new(),
                draw_sources: Vec::new(),
                cleanup_discards: Vec::new(),
                triggered_this_turn: Vec::new(),
                step_sources: Vec::new(),
                life_sources: Vec::new(),
                counter_sources: Vec::new(),
                death_sources: Vec::new(),
                ability_target_sources: None,
                attachment_sources: Vec::new(),
                exile_links: Vec::new(),
                exile_permissions: Vec::new(),
                hand_play_bans: None,
                borrowed_links: None,
                borrowed_uses: None,
                used_free_hand_permissions: None,
                graveyard_cast_permissions: None,
                turn_cast_grants: None,
                graveyard_type_uses: None,
                resolution_free_casts: None,
                miracle: None,
                temporary_abilities: None,
                self_granted_abilities: None,
                suspected: None,
                death_exile_replacements: None,
                roles: None,
                hand_size: None,
                preparations: None,
                adventures: None,
                plot: None,
                warp: None,
                known_hand: Vec::new(),
                explore_data: None,
                evidence_data: None,
                delayed_sacrifices: None,
                delayed_returns: None,
                mobilize: None,
                saddled: Vec::new(),
                saddle_payment: None,
                crew: None,
                last_target_library_move: None,
                players,
                rng,
                active: config.starting_player,
                priority: config.starting_player,
                starting: config.starting_player,
                turn: 1,
                step: Step::Pregame,
                pending: Pending::Mulligan(config.starting_player),
                sba_batch: None,
                entry_batch: None,
                stack: Vec::new(),
                resolving: None,
                announcing: None,
                mana_announcement_stack: None,
                pending_triggers: Vec::new(),
                ordering_triggers: Vec::new(),
                next_stack_id: 0,
                passes: 0,
                mulligan_round: [false; 2],
                land_played: false,
                continuous_land_plays_used: 0,
                attack_declared: false,
                attackers: Vec::new(),
                blocking: Vec::new(),
                blocks: Vec::new(),
                selected_attackers: Vec::new(),
                selected_blocks: Vec::new(),
                hits: Vec::new(),
                damage_choices: Vec::new(),
                first_strikers: Vec::new(),
                outcome: None,
            },
        };
        for p in [Player::P0, Player::P1] {
            for _ in 0..7 {
                game.draw(p);
            }
        }
        if game
            .state
            .objects
            .iter()
            .any(|o| o.card.zone_replacement().is_some())
        {
            game.state.zone_rules = Some(Box::default());
        }
        Ok(game)
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn outcome(&self) -> Option<Outcome> {
        self.state.outcome
    }
    pub fn snapshot(&self) -> Snapshot {
        Snapshot(self.state.clone())
    }
    pub fn restore(&mut self, snapshot: &Snapshot) -> Result<(), Error> {
        let revision = self.revision.checked_add(1).ok_or(Error::CapacityExceeded(
            crate::CapacityExceeded {
                resource: "revision exhausted",
            },
        ))?;
        self.state.clone_from(&snapshot.0);
        self.revision = revision;
        Ok(())
    }

    /// Player whose game decisions must currently be made. Resources and
    /// object controllers continue to belong to this semantic player.
    pub fn semantic_actor(&self) -> Option<Player> {
        match self.state.pending {
            Pending::OpeningHand { player: p, .. }
            | Pending::OptionalSearch(p)
            | Pending::Mulligan(p)
            | Pending::Bottom { player: p, .. } => Some(p),
            Pending::Priority => Some(self.state.priority),
            Pending::MiracleReveal { player }
            | Pending::ResolutionCasts { player }
            | Pending::ZoneReplacement { player }
            | Pending::ManaLotSpend { player }
            | Pending::Replacement { player }
            | Pending::CreatedToken { player }
            | Pending::PlotPayment { player }
            | Pending::EvidenceIntent { player }
            | Pending::Evidence { player }
            | Pending::Endure { player, .. }
            | Pending::Convoke { player }
            | Pending::Improvise { player }
            | Pending::Mobilize { player }
            | Pending::TeamworkIntent { player }
            | Pending::TeamworkPayment { player }
            | Pending::Saddle { player }
            | Pending::Crew { player }
            | Pending::Explore { player }
            | Pending::Bargain { player }
            | Pending::ResolutionX { player }
            | Pending::SourceCounterAmount { player }
            | Pending::X { player }
            | Pending::Casualty { player }
            | Pending::ManaRollback { player }
            | Pending::CastCosts { player }
            | Pending::CasualtyPayment { player, .. }
            | Pending::Kicker { player }
            | Pending::Gift { player }
            | Pending::Offspring { player }
            | Pending::SpellCost { player }
            | Pending::ActivationCost { player }
            | Pending::ConniveOrder { player }
            | Pending::ResolutionLife { player }
            | Pending::ResolutionMana { player }
            | Pending::AdditionalCost { player }
            | Pending::EntryTimestamp { player }
            | Pending::EntryReplacement { player }
            | Pending::LinkedChoice { player }
            | Pending::EntryValue { player }
            | Pending::LandCardName { player }
            | Pending::EntryLife { player }
            | Pending::AuraEntry { player }
            | Pending::TokenChoice { player, .. }
            | Pending::EffectChoice { player }
            | Pending::HandDefender { player }
            | Pending::Role { player }
            | Pending::Legend { player }
            | Pending::MilledSelection { player }
            | Pending::HandCardChoice { player }
            | Pending::RevealedDiscard { player, .. }
            | Pending::RecoverMilled { player, .. }
            | Pending::EffectSearch { player }
            | Pending::LibraryGraveyardSearch { player }
            | Pending::EquipmentAttachment { player }
            | Pending::EffectLibrarySelection { player }
            | Pending::EffectLibrary { player }
            | Pending::EffectMana { player, .. }
            | Pending::PayCosts { player }
            | Pending::Modes { player }
            | Pending::Retarget { player }
            | Pending::TargetAmount { player }
            | Pending::Targets { player }
            | Pending::OrderTriggers { player }
            | Pending::TargetLibrary { player }
            | Pending::ManaValueParity { player }
            | Pending::ForageMethod { player }
            | Pending::Forage { player }
            | Pending::EffectOptional { player, .. }
            | Pending::EffectDiscard { player, .. } => Some(player),
            Pending::Attackers | Pending::Damage | Pending::Discard => Some(self.state.active),
            Pending::Blockers => Some(self.state.active.other()),
            Pending::Terminal => None,
        }
    }

    /// External decision maker, including a player controlling another player.
    pub fn actor(&self) -> Option<Player> {
        self.semantic_actor().map(|p| self.decision_controller(p))
    }

    /// Complete legal space, including damage assignments and scalar-X intervals.
    pub fn legal_action_space(&self) -> LegalActionSpace {
        LegalActionSpace {
            source_counter_choice: self.source_counter_choice(),
            mana_lot_spend: self.mana_lot_spend_range(),
            mana_payment: self.mana_payment_domain(),
            choices: self.legal_actions(),
            x_choice: self.x_choice(),
            damage_assignment: self.damage_assignment_range(),
        }
    }
    /// Exact bounds without allocating or enumerating numeric damage amounts.
    pub fn damage_assignment_range(&self) -> Option<DamageAssignmentRange> {
        if self.state.pending != Pending::Damage {
            return None;
        }
        let choice = self.state.damage_choices.last().expect("damage choice");
        let minimum = if choice.blockers.len() == 1 && choice.trample {
            if choice.prior_blockers_lethal {
                self.lethal_assignment(choice.source, choice.blockers[0])
                    .min(choice.remaining)
            } else {
                choice.remaining
            }
        } else {
            0
        };
        Some(DamageAssignmentRange {
            source: choice.source.reference,
            target: Target::Card(choice.blockers[0]),
            minimum,
            maximum: choice.remaining,
        })
    }
    /// Reuse a caller-owned buffer of concrete indexed choices. Numeric ranges
    /// up to 64 wide are exhaustive; larger damage/X domains provide bounded
    /// representatives. Use legal_action_space for the complete parametric space.
    pub fn legal_actions_into(&self, out: &mut Vec<Action>) {
        out.clear();
        if self.semantic_actor().is_none() {
            return;
        }
        self.visit_actions(|a| {
            out.push(a);
            false
        });
    }

    /// Concrete indexed choices; large numeric domains are represented completely
    /// by legal_action_space(), not enumerated in this vector.
    pub fn legal_actions(&self) -> Vec<Action> {
        let mut actions = Vec::new();
        self.legal_actions_into(&mut actions);
        actions
    }

    /// Mutation happens only after complete validation. Rejected actions leave state
    /// and revision unchanged. No user-supplied IDs are used to index before validation.
    pub fn apply(&mut self, actor: Player, action: Action) -> Result<(), Error> {
        // Conceding is always the caller's own choice, even while controlled.
        if action != Action::Concede && self.state.outcome.is_none() && self.actor() != Some(actor)
        {
            return Err(Error::WrongActor);
        }
        let actor = if action == Action::Concede {
            actor
        } else {
            self.semantic_actor().unwrap_or(actor)
        };
        let deferred_action = self.state.mana_lots.is_some().then(|| action.clone());
        // Cold mechanic presence avoids an extra action clone in ordinary games.
        let zone_action = self.state.zone_rules.is_some().then(|| action.clone());
        let checkpoint = self.state.clone();
        let revision = self.revision;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if matches!(self.state.pending, Pending::ManaLotSpend { .. }) {
                self.apply_mana_lot_choice(actor, action)?;
            } else {
                self.apply_transaction(actor, action)?;
            }
            // Suspended programs defer SBAs, but every committed observation must
            // still have representable derived values. Ordinary resolutions have
            // already checked battlefield stats in the SBA pass.
            if (self.state.resolving.is_some()
                || self.miracle_choice_pending()
                || self.plot_payment().is_some()
                || self.state.announcing.is_some()
                || self.state.entry_batch.is_some())
                && (!self.state.objects.ptr_eq(&checkpoint.objects)
                    || !live_counters::same_storage(
                        &self.state.named_counters,
                        &checkpoint.named_counters,
                    )
                    || self.state.active != checkpoint.active
                    || self.state.attackers != checkpoint.attackers
                    || self.state.static_sources != checkpoint.static_sources
                    || self
                        .state
                        .players
                        .iter()
                        .zip(&checkpoint.players)
                        .any(|(now, old)| {
                            now.life != old.life
                                || (now.life_gained_this_turn > 0)
                                    != (old.life_gained_this_turn > 0)
                        }))
            {
                for (id, object) in self.state.objects.iter().enumerate() {
                    if object.zone == Zone::Battlefield {
                        let _ = self.stats(id as CardId);
                    }
                }
            }
            Ok(())
        }));
        match result {
            Ok(Ok(())) => {
                if let Some(action) = deferred_action {
                    self.defer_mana_allocation(actor, action, checkpoint, revision);
                }
                Ok(())
            }
            Ok(Err(error)) => {
                self.state = checkpoint;
                self.revision = revision;
                Err(error)
            }
            Err(payload) => match payload.downcast::<zone_replacements::Request>() {
                Ok(request) => {
                    if self
                        .state
                        .mana_lots
                        .as_ref()
                        .is_some_and(|r| r.trace.is_some())
                    {
                        let saved = checkpoint.clone();
                        self.defer_mana_allocation(
                            actor,
                            zone_action.clone().expect("zone action"),
                            checkpoint,
                            revision,
                        );
                        if matches!(self.state.pending, Pending::ManaLotSpend { .. }) {
                            return Ok(());
                        }
                        return self.suspend_zone_replacement(
                            *request,
                            saved,
                            actor,
                            zone_action,
                            revision,
                        );
                    }
                    self.suspend_zone_replacement(
                        *request,
                        checkpoint,
                        actor,
                        zone_action,
                        revision,
                    )
                }
                Err(payload) => {
                    self.state = checkpoint;
                    self.revision = revision;
                    match payload.downcast::<CapacitySignal>() {
                        Ok(signal) => Err(Error::CapacityExceeded(signal.0)),
                        Err(payload) => std::panic::resume_unwind(payload),
                    }
                }
            },
        }
    }
    fn apply_transaction(&mut self, actor: Player, action: Action) -> Result<(), Error> {
        if self.state.outcome.is_some() {
            return Err(Error::GameOver);
        }
        if action == Action::Concede {
            self.state.outcome = Some(Outcome::Winner(actor.other()));
            self.state.pending = Pending::Terminal;
        } else {
            if self.semantic_actor() != Some(actor) {
                return Err(Error::WrongActor);
            }
            let legal = if let Action::PayCosts { payment } = &action {
                self.mana_payment_domain()
                    .is_some_and(|d| d.contains(*payment))
                    && self.mana_parent_fundable(
                        self.state.announcing.as_ref().and_then(|a| a.additional),
                        Some(*payment),
                    )
            } else if let Action::ChooseSourceCounterAmount { amount } = &action {
                self.source_counter_choice()
                    .is_some_and(|domain| *amount <= domain.maximum)
            } else if let Action::ChooseX { value } = &action {
                self.x_choice()
                    .is_some_and(|domain| domain.contains(*value))
            } else if let Action::AssignDamage { target, amount } = &action {
                self.damage_assignment_range().is_some_and(|range| {
                    *target == range.target && *amount >= range.minimum && *amount <= range.maximum
                })
            } else {
                self.visit_actions(|a| a == action)
            };
            if !legal {
                return Err(Error::IllegalAction);
            }
            self.prepare_cast_rollback(actor, &action);
            self.capture_cast_mana_action(actor, &action);
            self.clear_zone_reveals();
            self.prepare_mana_allocations(&action);
            self.prepare_library_visibility(&action);
            self.apply_legal(actor, action);
            // Resolution is atomic with respect to SBAs and priority, even when
            // a program is suspended for an optional effect or discard choice.
            loop {
                if self.miracle_choice_pending() {
                    break;
                }
                if self.advance_replacements(true) {
                    break;
                }
                if self.advance_entries() {
                    break;
                }
                if self.state.resolving.is_none()
                    && self.state.replacements.is_none()
                    && self.state.announcing.is_none()
                    && self.plot_payment().is_none()
                {
                    self.state_based_actions();
                    if self.state.entry_batch.is_some() {
                        continue;
                    }
                    if self.continue_ending_turn() {
                        break;
                    }
                    self.prepare_triggers();
                }
                break;
            }
        }
        self.finish_cast_mana_action();
        if self.foreign_cast_choice_pending()
            && !(self
                .state
                .cast_rollback
                .as_ref()
                .is_some_and(|r| r.capture.is_some())
                && (self.state.entry_batch.is_some() || self.state.replacements.is_some()))
        {
            let fundable = self.current_cast_costs_fundable();
            self.finish_cast_rollback(fundable);
        }
        self.finish_library_visibility();
        self.clear_finished_cast_rollback();
        self.refresh_copy_grant_availability();
        self.prune_modal_history();
        self.revision = self.revision.checked_add(1).capacity("revision exhausted");
        Ok(())
    }

    pub fn apply_index(&mut self, actor: Player, revision: u64, index: usize) -> Result<(), Error> {
        if revision != self.revision {
            return Err(Error::StaleRevision);
        }
        if self.actor() != Some(actor) {
            return Err(Error::WrongActor);
        }
        let mut n = 0;
        let mut found = None;
        self.visit_actions(|a| {
            if n == index {
                found = Some(a);
                true
            } else {
                n += 1;
                false
            }
        });
        self.apply(actor, found.ok_or(Error::IllegalAction)?)
    }

    fn visit_actions(&self, mut emit: impl FnMut(Action) -> bool) -> bool {
        if self.state.entry_batch.is_some() {
            return self.visit_unfiltered_actions(|action| {
                self.mana_action_payable(&action)
                    && self.cast_entry_action_fundable(&action)
                    && emit(action)
            });
        }
        if self.state.mana_lots.is_none() {
            return self.visit_unfiltered_actions(emit);
        }
        self.visit_unfiltered_actions(|action| self.mana_action_payable(&action) && emit(action))
    }
    fn visit_unfiltered_actions(&self, mut emit: impl FnMut(Action) -> bool) -> bool {
        let s = &self.state;
        let Some(actor) = self.semantic_actor() else {
            return false;
        };
        if emit(Action::Concede) {
            return true;
        }
        // This query is read-only: identical fixed spell costs share a funding
        // result. Cache capacity affects work only, never the admitted action set.
        let mut spell_funding_cache: [Option<(Mana, u32, Card, bool)>; 8] = [None; 8];
        match s.pending {
            Pending::LibraryGraveyardSearch { .. } => return self.visit_zone_search(&mut emit),
            Pending::ZoneReplacement { .. } => {
                if self.zone_choice_actions(&mut emit) {
                    return true;
                }
            }
            Pending::ManaLotSpend { .. } => {
                if let Some(range) = self.mana_lot_spend_range() {
                    if let Some(values) = &range.values {
                        for &amount in values.iter().take(64) {
                            if emit(Action::ChooseManaLotSpend { amount }) {
                                return true;
                            }
                        }
                    } else if range.maximum - range.minimum <= 64 {
                        for amount in range.minimum..=range.maximum {
                            if emit(Action::ChooseManaLotSpend { amount }) {
                                return true;
                            }
                        }
                    } else {
                        for amount in [
                            range.minimum,
                            range.minimum + (range.maximum - range.minimum) / 2,
                            range.maximum,
                        ] {
                            if emit(Action::ChooseManaLotSpend { amount }) {
                                return true;
                            }
                        }
                    }
                }
            }
            Pending::Replacement { .. } => {
                if self.visit_replacement_actions(&mut emit) {
                    return true;
                }
            }
            Pending::CreatedToken { .. } => {
                if self.visit_created_token_actions(&mut emit) {
                    return true;
                }
            }
            Pending::Endure { .. } => {
                if emit(Action::ChooseEndure { put_counters: true })
                    || emit(Action::ChooseEndure {
                        put_counters: false,
                    })
                {
                    return true;
                }
            }
            Pending::Convoke { .. } | Pending::Improvise { .. } => {
                if self.visit_convoke_actions(&mut emit) {
                    return true;
                }
            }
            Pending::Mobilize { .. } => {
                if self.visit_mobilize_actions(&mut emit) {
                    return true;
                }
            }
            Pending::Crew { .. } => {
                if self.visit_crew_actions(&mut emit) {
                    return true;
                }
            }
            Pending::TeamworkIntent { .. } => {
                for pay in [false, true] {
                    if self.teamwork_intent_fundable(pay) && emit(Action::ChooseTeamwork { pay }) {
                        return true;
                    }
                }
            }
            Pending::TeamworkPayment { .. } => {
                return self.visit_teamwork_payments(&mut emit);
            }
            Pending::Saddle { .. } => {
                if self.visit_saddle_actions(&mut emit) {
                    return true;
                }
            }
            Pending::ManaValueParity { .. } => {
                for parity in [ManaValueParity::Odd, ManaValueParity::Even] {
                    if emit(Action::ChooseManaValueParity { parity }) {
                        return true;
                    }
                }
            }
            Pending::TargetLibrary { .. } => {
                let object = s
                    .resolving
                    .as_ref()
                    .unwrap()
                    .target_library
                    .as_ref()
                    .unwrap()
                    .targets[0]
                    .object
                    .unwrap();
                for destination in s
                    .resolving
                    .as_ref()
                    .unwrap()
                    .target_library
                    .as_ref()
                    .unwrap()
                    .destinations()
                {
                    if emit(Action::ChooseLibraryDestination {
                        object,
                        destination,
                    }) {
                        return true;
                    }
                }
            }
            Pending::EvidenceIntent { .. } => {
                for collect in [false, true] {
                    if self.evidence_intent_fundable(collect)
                        && emit(Action::ChooseCollectEvidence { collect })
                    {
                        return true;
                    }
                }
            }
            Pending::Evidence { .. } => {
                if self.visit_evidence_actions(&mut emit) {
                    return true;
                }
            }
            Pending::ForageMethod { .. } | Pending::Forage { .. } => {
                if self.visit_forage_actions(&mut emit) {
                    return true;
                }
            }
            Pending::ResolutionLife { .. } => {
                if emit(Action::DeclineResolutionLife)
                    || self.can_pay_resolution_life() && emit(Action::PayResolutionLife)
                {
                    return true;
                }
            }
            Pending::ResolutionMana { .. } => {
                if self.visit_resolution_payment(&mut emit) {
                    return true;
                }
            }
            Pending::EntryTimestamp { .. } => {
                for object in self.entry_timestamp_candidates() {
                    if emit(Action::ChooseEntryTimestamp { object }) {
                        return true;
                    }
                }
            }
            Pending::EntryReplacement { .. } => {
                for effect in self.entry_tap_candidates() {
                    if emit(Action::ChooseEntryReplacement {
                        source: effect.source,
                        intrinsic: effect.intrinsic,
                    }) {
                        return true;
                    }
                }
            }
            Pending::LinkedChoice { .. } => {
                if self.visit_linked_choices(&mut emit) {
                    return true;
                }
            }
            Pending::EntryValue { .. } | Pending::LandCardName { .. } => {
                if self.visit_entry_values(|value| emit(Action::ChooseEntryValue { value })) {
                    return true;
                }
            }
            Pending::EntryLife { .. } => {
                if emit(Action::ChooseEntryLife { pay: false })
                    || self.can_pay_entry_life() && emit(Action::ChooseEntryLife { pay: true })
                {
                    return true;
                }
            }
            Pending::AuraEntry { .. } => {
                for (id, _) in s.objects.iter().enumerate() {
                    let object = self.object_ref(id as CardId);
                    if self.entry_candidate(object) && emit(Action::ChooseAuraAttachment { object })
                    {
                        return true;
                    }
                }
            }
            Pending::HandDefender { .. } => {
                if self.visit_hand_defenders(&mut emit) {
                    return true;
                }
            }
            Pending::TokenChoice { tokens, .. } => {
                for &token in tokens {
                    if emit(Action::ChooseToken { token }) {
                        return true;
                    }
                }
            }
            Pending::EffectChoice { player } => {
                let choice = s.resolving.as_ref().unwrap().choice.as_ref().unwrap();
                if choice.kind == EffectChoiceKind::RemoveCreatureCounters(1) {
                    return self.visit_resolution_counter_payments(player, &mut emit);
                }
                if matches!(
                    choice.kind,
                    EffectChoiceKind::LandCards { .. }
                        | EffectChoiceKind::UntapLands { .. }
                        | EffectChoiceKind::DiscardedLand { .. }
                ) && emit(Action::FinishLandSelection)
                {
                    return true;
                }
                if choice.skip_to.is_some()
                    && !matches!(
                        choice.kind,
                        EffectChoiceKind::LandCards { .. }
                            | EffectChoiceKind::UntapLands { .. }
                            | EffectChoiceKind::DiscardedLand { .. }
                    )
                    && emit(
                        if matches!(
                            choice.kind,
                            EffectChoiceKind::HandExileCopy
                                | EffectChoiceKind::HandCreature
                                | EffectChoiceKind::HandPermanent { .. }
                                | EffectChoiceKind::HandLand { .. }
                        ) {
                            Action::DeclineHandEntry
                        } else {
                            Action::DeclineEffectPayment
                        },
                    )
                {
                    return true;
                }
                for (index, _) in s.objects.iter().enumerate() {
                    let object = self.object_ref(index as CardId);
                    if self.effect_choice_candidate(choice, player, object) {
                        let action = match choice.kind {
                            EffectChoiceKind::HandExileCopy | EffectChoiceKind::HandCreature => {
                                Action::PutHandCreature { object }
                            }
                            EffectChoiceKind::HandPermanent { .. } => {
                                Action::PutHandPermanent { object }
                            }
                            EffectChoiceKind::HandLand { .. } => Action::PutHandLand { object },
                            EffectChoiceKind::LandCards { .. }
                            | EffectChoiceKind::DiscardedLand { .. }
                            | EffectChoiceKind::UntapLands { .. } => Action::SelectLand { object },
                            EffectChoiceKind::Sacrifice
                            | EffectChoiceKind::ExileGreatestManaValue
                            | EffectChoiceKind::ExileGraveyard
                            | EffectChoiceKind::TapPermanents(_)
                            | EffectChoiceKind::HandToTop => Action::Sacrifice { object },
                            EffectChoiceKind::Discard => Action::Discard { card: object.card },
                            EffectChoiceKind::SacrificeOrDiscard { .. } => {
                                if s.objects[index].zone == Zone::Hand {
                                    Action::Discard { card: object.card }
                                } else {
                                    Action::Sacrifice { object }
                                }
                            }
                            _ => unreachable!("forage has its own choice"),
                        };
                        if emit(action) {
                            return true;
                        }
                    }
                }
            }
            Pending::Role { player } => {
                for group in &s.sba_batch.as_ref().unwrap().roles {
                    if group.controller == player && group.keep.is_none() {
                        for &object in &group.objects {
                            if emit(Action::KeepRole { object }) {
                                return true;
                            }
                        }
                    }
                }
            }
            Pending::Legend { player } => {
                for group in &s.sba_batch.as_ref().unwrap().groups {
                    if group.controller == player && group.keep.is_none() {
                        for &object in &group.objects {
                            if emit(Action::KeepLegend { object }) {
                                return true;
                            }
                        }
                    }
                }
            }
            Pending::AdditionalCost { .. } => {
                if self.graveyard_symbol_selection_complete()
                    && emit(Action::FinishGraveyardSymbolCost)
                {
                    return true;
                }
                if self
                    .state
                    .announcing
                    .as_ref()
                    .unwrap()
                    .graveyard_origin_counter_cost()
                    .is_some()
                {
                    return self.visit_creature_counter_cost(&mut emit);
                }
                for (id, _) in s.objects.iter().enumerate() {
                    let object = self.object_ref(id as CardId);
                    if self.additional_choice_available(object)
                        && emit(Action::ChooseAdditionalCost { object })
                    {
                        return true;
                    }
                }
            }
            Pending::PlotPayment { .. } => {
                if self.visit_plot_payment(&mut emit) {
                    return true;
                }
            }
            Pending::PayCosts { .. } => {
                if self.visit_cost_payment(&mut emit) {
                    return true;
                }
            }
            Pending::EffectMana { production, .. } => {
                if production.visit(|production| emit(Action::ChooseMana { production })) {
                    return true;
                }
            }
            Pending::OpeningHand {
                player,
                ref eligible,
            } => {
                if emit(Action::FinishOpeningHand) {
                    return true;
                }
                for &object in &eligible[player.index()] {
                    if self.object_ref(object.card) == object
                        && s.objects[object.card as usize].zone == Zone::Hand
                        && emit(Action::BeginOnBattlefield { object })
                    {
                        return true;
                    }
                }
            }
            Pending::OptionalSearch(_) => {
                if emit(Action::ChooseOptional { accept: true })
                    || emit(Action::ChooseOptional { accept: false })
                {
                    return true;
                }
            }
            Pending::Mulligan(p) => {
                if emit(Action::Keep) {
                    return true;
                }
                if s.players[p.index()].mulligans < 7 && emit(Action::Mulligan) {
                    return true;
                }
            }
            Pending::Bottom { .. } | Pending::Discard | Pending::EffectDiscard { .. } => {
                for (i, obj) in s.objects.iter().enumerate() {
                    if obj.owner == actor && obj.zone == Zone::Hand && !obj.is_token() {
                        if matches!(s.pending, Pending::EffectDiscard { .. })
                            && s.resolving.as_ref().unwrap().selected_discards[actor.index()]
                                .contains(&(i as CardId))
                        {
                            continue;
                        }
                        if s.pending == Pending::Discard
                            && s.cleanup_discards.contains(&(i as CardId))
                        {
                            continue;
                        }
                        let a = if matches!(s.pending, Pending::Bottom { .. }) {
                            Action::Bottom { card: i as CardId }
                        } else {
                            Action::Discard { card: i as CardId }
                        };
                        if emit(a) {
                            return true;
                        }
                    }
                }
            }
            Pending::MiracleReveal { player } => {
                if self
                    .miracle_choice_view(player)
                    .is_some_and(|v| v.can_reveal)
                    && emit(Action::RevealMiracle)
                {
                    return true;
                }
                if emit(Action::DeclineMiracle) {
                    return true;
                }
            }
            Pending::ResolutionCasts { player } => {
                if self.visit_resolution_casts(player, &mut emit) {
                    return true;
                }
            }
            Pending::Priority => {
                if emit(Action::Pass) {
                    return true;
                }
                let sorcery = actor == s.active
                    && matches!(s.step, Step::Main1 | Step::Main2)
                    && s.stack.is_empty();
                let casting_prohibited =
                    s.casting_prohibited || self.static_casting_prohibited(actor);
                let free_hand_cast = self.free_hand_cast_available(actor);
                let limited_hand_casts = self.limited_free_hand_permissions(actor);
                if self.visit_graveyard_permanent_plays(
                    actor,
                    sorcery,
                    casting_prohibited,
                    &mut emit,
                ) {
                    return true;
                }
                for (i, obj) in s.objects.iter().enumerate() {
                    let card = i as CardId;
                    let authorized_origin = match obj.zone {
                        Zone::Battlefield => obj.controller == actor,
                        Zone::Hand | Zone::Graveyard => obj.owner == actor,
                        Zone::Library => {
                            self.can_cast_library_top(actor, card)
                                || self.can_play_static_land(actor, card)
                                || self.turn_free_library_origin(actor, card)
                        }
                        Zone::Exile => {
                            self.can_play_exiled(actor, card)
                                || self.plotted_cast_available(actor, card)
                        }
                        _ => false,
                    };
                    if !authorized_origin {
                        continue;
                    }
                    if obj.zone == Zone::Battlefield
                        && !ability_world::CharacteristicWorld::activation_prohibited(
                            self, actor, card,
                        )
                        && self.printed_abilities_available(card)
                        && !obj.tapped
                        && ability_world::CharacteristicWorld::intrinsic_mana_color(self, card)
                            .is_some()
                        && (!self.is_creature(card)
                            || !obj.summoning_sick
                            || ability_world::CharacteristicWorld::activation_keywords(self, card)
                                .contains(Keywords::HASTE))
                        && emit(Action::TapMana { card })
                    {
                        return true;
                    }
                    if obj.zone == Zone::Battlefield
                        && self.visit_granted_mana(card, actor, &mut emit)
                    {
                        return true;
                    }
                    if obj.zone == Zone::Battlefield
                        && self.visit_borrowed_activations(card, actor, sorcery, &mut emit)
                    {
                        return true;
                    }
                    if obj.zone == Zone::Battlefield
                        && self.visit_self_granted_activations(card, actor, sorcery, &mut emit)
                    {
                        return true;
                    }
                    if obj.zone == Zone::Battlefield
                        && self.visit_attached_activations(card, actor, &mut emit)
                    {
                        return true;
                    }
                    if matches!(obj.zone, Zone::Battlefield | Zone::Hand | Zone::Graveyard)
                        && self.visit_activations(card, actor, sorcery, &mut emit)
                    {
                        return true;
                    }
                    if sorcery
                        && obj.zone == Zone::Hand
                        && !obj.is_token()
                        && obj
                            .card
                            .plot_cost()
                            .is_some_and(|cost| self.plot_can_fund(actor, cost))
                        && emit(Action::AnnouncePlot { card })
                    {
                        return true;
                    }
                    if obj.zone == Zone::Hand && self.hand_play_prohibited() {
                        continue;
                    }
                    if casting_prohibited {
                        if obj.card.is_land()
                            && sorcery
                            && (obj.zone == Zone::Hand
                                || self.can_play_exiled_land(actor, card)
                                || self.has_graveyard_cast(actor, card, GraveyardCastKind::Play)
                                || self.can_play_static_land(actor, card))
                            && (!s.land_played
                                || self
                                    .continuous_land_plays(actor)
                                    .checked_add(s.players[actor.index()].extra_land_plays_granted)
                                    .capacity("land allowance exhausted")
                                    > s.continuous_land_plays_used)
                            && emit(Action::PlayLand { card })
                        {
                            return true;
                        }
                        // Plot and activated abilities were offered above; every
                        // casting method/face below remains prohibited.
                        continue;
                    }
                    if sorcery
                        && obj.zone == Zone::Exile
                        && self.plotted_cast_available(actor, card)
                        && emit(Action::AnnouncePlotted { card })
                    {
                        return true;
                    }
                    if obj.zone == Zone::Hand
                        && (sorcery
                            || obj.card.is_instant()
                            || self.keywords(card).contains(Keywords::FLASH)
                            || self.cast_timing_granted(actor, obj.card))
                        && obj
                            .card
                            .warp_cost()
                            .is_some_and(|cost| self.warp_cast_fundable(actor, card, cost))
                        && emit(Action::AnnounceWarp { card })
                    {
                        return true;
                    }
                    if self.sneak_cast_available(actor, card)
                        && emit(Action::AnnounceSneak {
                            card,
                            permission: None,
                        })
                    {
                        return true;
                    }
                    if obj.card.adventure_spell().is_some()
                        && self.alternate_cast_available(
                            actor,
                            card,
                            sorcery,
                            obj.card.adventure_spell().unwrap(),
                        )
                        && emit(Action::AnnounceAdventure { card })
                    {
                        return true;
                    }
                    if obj.card.back_face().is_some()
                        && self.back_cast_available(actor, card, sorcery)
                        && emit(Action::AnnounceBack { card })
                    {
                        return true;
                    }
                    if let Some(definition) = obj.card.omen_spell()
                        && self.alternate_cast_available(actor, card, sorcery, definition)
                        && emit(Action::AnnounceOmen { card })
                    {
                        return true;
                    }
                    if !casting_prohibited
                        && self.visit_turn_free_casts(actor, card, sorcery, &mut emit)
                    {
                        return true;
                    }
                    if !casting_prohibited && !obj.is_token() && !obj.card.is_land() {
                        if free_hand_cast && obj.zone == Zone::Hand {
                            for face in [
                                CastFace::Normal,
                                CastFace::Adventure,
                                CastFace::Omen,
                                CastFace::Back,
                            ] {
                                for flash_surcharge in [false, true] {
                                    if self.permitted_cast_available(
                                        actor,
                                        card,
                                        CastMethod::Free,
                                        face,
                                        sorcery,
                                        flash_surcharge,
                                    ) && emit(Action::AnnounceFree {
                                        card,
                                        face,
                                        flash_surcharge,
                                    }) {
                                        return true;
                                    }
                                }
                            }
                        }
                        if obj.zone == Zone::Hand && !limited_hand_casts.is_empty() {
                            for face in [
                                CastFace::Normal,
                                CastFace::Adventure,
                                CastFace::Omen,
                                CastFace::Back,
                            ] {
                                let definition = match face {
                                    CastFace::Normal => Some(obj.card),
                                    CastFace::Adventure => obj.card.adventure_spell(),
                                    CastFace::Omen => obj.card.omen_spell(),
                                    CastFace::Back => obj.card.back_face(),
                                };
                                if !definition.is_some_and(|d| {
                                    crate::cards::CastFilter::InstantSorcery.matches(d)
                                }) {
                                    continue;
                                }
                                for flash_surcharge in [false, true] {
                                    if self.permitted_cast_available(
                                        actor,
                                        card,
                                        CastMethod::Free,
                                        face,
                                        sorcery,
                                        flash_surcharge,
                                    ) {
                                        for &permission in &limited_hand_casts {
                                            if emit(Action::AnnounceFreeFromPermission {
                                                card,
                                                face,
                                                flash_surcharge,
                                                permission,
                                            }) {
                                                return true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if obj.zone == Zone::Graveyard
                            && obj.card.graveyard_counter_cost().is_some()
                            && self.permitted_cast_available(
                                actor,
                                card,
                                CastMethod::GraveyardCounterCost,
                                CastFace::Normal,
                                sorcery,
                                false,
                            )
                            && emit(Action::AnnounceCounterGraveyard { card })
                        {
                            return true;
                        }
                        if !obj.card.is_land()
                            && self.has_graveyard_cast(actor, card, GraveyardCastKind::Cast)
                            && self.permitted_cast_available(
                                actor,
                                card,
                                CastMethod::Normal,
                                CastFace::Normal,
                                sorcery,
                                false,
                            )
                            && emit(Action::AnnounceCast { card })
                        {
                            return true;
                        }
                        if self.has_graveyard_cast(actor, card, GraveyardCastKind::Flashback) {
                            for flash_surcharge in [false, true] {
                                if self.permitted_cast_available(
                                    actor,
                                    card,
                                    CastMethod::GrantedFlashback,
                                    CastFace::Normal,
                                    sorcery,
                                    flash_surcharge,
                                ) && emit(Action::AnnounceGrantedFlashback {
                                    card,
                                    flash_surcharge,
                                }) {
                                    return true;
                                }
                            }
                        }
                    }
                    let flashback =
                        obj.zone == Zone::Graveyard && obj.card.flashback_cost().is_some();
                    if flashback && obj.card.flashback_additional().is_some() {
                        if self.permitted_cast_available(
                            actor,
                            card,
                            CastMethod::Flashback,
                            CastFace::Normal,
                            sorcery,
                            false,
                        ) && emit(Action::AnnounceFlashback { card })
                        {
                            return true;
                        }
                        continue;
                    }
                    let exile = obj.zone == Zone::Exile && self.can_play_exiled(actor, card);
                    let library = self.can_cast_library_top(actor, card)
                        || (obj.zone == Zone::Library && self.can_play_static_land(actor, card));
                    let graveyard_land = obj.card.is_land()
                        && (self.has_graveyard_cast(actor, card, GraveyardCastKind::Play)
                            || (obj.zone == Zone::Graveyard
                                && self.can_play_static_land(actor, card)));
                    if (obj.zone != Zone::Hand
                        && !flashback
                        && !exile
                        && !library
                        && !graveyard_land)
                        || obj.is_token()
                    {
                        continue;
                    }
                    if obj.card.is_land() {
                        if obj.zone == Zone::Exile && !self.can_play_exiled_land(actor, card) {
                            continue;
                        }
                        if sorcery
                            && (!s.land_played
                                || self
                                    .continuous_land_plays(actor)
                                    .checked_add(s.players[actor.index()].extra_land_plays_granted)
                                    .capacity("land allowance exhausted")
                                    > s.continuous_land_plays_used)
                            && emit(Action::PlayLand { card })
                        {
                            return true;
                        }
                        continue;
                    }
                    if obj.card.flash_surcharge().is_some()
                        && self.flash_surcharge_cast_fundable(actor, card, flashback)
                        && emit(Action::AnnounceFlashSurcharge { card })
                    {
                        return true;
                    }
                    if !sorcery
                        && !obj.card.is_instant()
                        && !self.keywords(card).contains(Keywords::FLASH)
                        && !self.cast_timing_granted(actor, obj.card)
                    {
                        continue;
                    }
                    let completable = match obj.card.effect() {
                        Effect::Modal(modes) => modes.iter().any(|mode| {
                            self.cast_targets_available(mode.targets, actor, self.object_ref(card))
                        }),
                        Effect::Spell(spec, _) | Effect::Aura(spec) => {
                            self.cast_targets_available(spec, actor, self.object_ref(card))
                                || obj.card.gift().is_some_and(|gift| {
                                    self.cast_targets_available(
                                        gift.promised_targets,
                                        actor,
                                        self.object_ref(card),
                                    )
                                })
                                || obj
                                    .card
                                    .teamwork()
                                    .and_then(|s| s.paid_targets)
                                    .is_some_and(|plan| {
                                        self.cast_targets_available(
                                            plan,
                                            actor,
                                            self.object_ref(card),
                                        )
                                    })
                                || self.kicked_cast_targets_available(actor, card)
                        }
                        _ => true,
                    };
                    if obj.card.x_symbols() > 0 {
                        // The currently typed printed flashback costs are fixed mana costs.
                        // Their alternative cost has no X, so the spell's declared X is zero.
                        if flashback {
                            if completable
                                && self.permitted_cast_available(
                                    actor,
                                    card,
                                    CastMethod::Flashback,
                                    CastFace::Normal,
                                    sorcery,
                                    false,
                                )
                                && emit(Action::AnnounceFlashback { card })
                            {
                                return true;
                            }
                            continue;
                        }
                        if completable
                            && !self.scalar_x_intervals(actor, card).is_empty()
                            && emit(if exile {
                                Action::AnnounceExile { card }
                            } else {
                                Action::AnnounceCast { card }
                            })
                        {
                            return true;
                        }
                        continue;
                    }
                    let target_cost = obj.card.target_cost_reduction().is_some();
                    let has_generic = if flashback {
                        obj.card.flashback_cost().unwrap().generic > 0
                    } else {
                        obj.card.cost().1 > 0
                    };
                    let reduction = if completable && has_generic {
                        self.spell_reduction(actor, card)
                    } else {
                        0
                    };
                    if completable {
                        let costs = if flashback {
                            std::iter::once(obj.card.flashback_cost().unwrap())
                                .chain([].iter().copied())
                        } else {
                            obj.card.cost_options()
                        };
                        let can_fund = costs
                            .into_iter()
                            .flat_map(|base| obj.card.additional_mana_options(base))
                            .any(|cost| {
                                if !self.spell_cost_option_available(actor, card, cost) {
                                    return false;
                                }
                                if obj.card.teamwork().is_some() {
                                    return self.teamwork_initial_fundable(actor, card, cost);
                                }
                                if obj.card.tiered_costs().is_some() {
                                    return self.tiered_initial_fundable(actor, card, cost);
                                }
                                if target_cost {
                                    return self.initial_target_cost_fundable(actor, card, cost);
                                }
                                if obj.card.collect_evidence_cost().is_some() {
                                    return self.evidence_cast_fundable(actor, card, cost, false)
                                        || self.evidence_cast_fundable(actor, card, cost, true);
                                }
                                if obj.card.has_bargain() {
                                    return self.initial_bargain_fundable(actor, card, cost);
                                }
                                // Object-specific mana permissions cannot share the
                                // printed-definition-only prefunding cache.
                                if self.foreign_stash(actor, card)
                                    || self.exile_cast_modifiers(actor, card).0
                                    || obj.card.has_convoke()
                                    || improvise::potential(self, obj.card)
                                    || obj.card.additional_cost().is_some()
                                {
                                    return self.spell_can_fund(actor, card, cost);
                                }
                                let colored = cost.colored;
                                let generic = cost.generic.saturating_sub(reduction);
                                let cached = spell_funding_cache
                                    .iter()
                                    .flatten()
                                    .find(|(required, amount, definition, _)| {
                                        *required == colored
                                            && *amount == generic
                                            && *definition == obj.card
                                    })
                                    .map(|(_, _, _, result)| *result);
                                if let Some(result) = cached {
                                    result
                                } else {
                                    let result = self.can_fund_for(
                                        FundingGoal {
                                            reserve: None,
                                            actor,
                                            required: colored,
                                            generic,
                                            source: None,
                                        },
                                        self.spell_mana_purpose_for_card(actor, card, obj.card),
                                    );
                                    if let Some(slot) =
                                        spell_funding_cache.iter_mut().find(|slot| slot.is_none())
                                    {
                                        *slot = Some((colored, generic, obj.card, result));
                                    }
                                    result
                                }
                            });
                        if can_fund
                            && emit(if flashback {
                                Action::AnnounceFlashback { card }
                            } else if exile {
                                Action::AnnounceExile { card }
                            } else {
                                Action::AnnounceCast { card }
                            })
                        {
                            return true;
                        }
                    }
                    if target_cost
                        && obj.zone == Zone::Hand
                        && obj.card.offspring_cost().is_none()
                        && obj.card.kicker_cost().is_none()
                        && obj.card.gift().is_none()
                        && obj.card.collect_evidence_cost().is_none()
                        && !self.has_casualty(actor, obj.card)
                        && !obj.card.has_bargain()
                        && obj.card.additional_cost().is_none()
                    {
                        let plan = match obj.card.effect() {
                            Effect::Spell(plan, _) | Effect::Aura(plan) => Some(plan),
                            _ => None,
                        };
                        if let Some(spec) = plan.and_then(|p| p.single()).filter(|s| s.max <= 1) {
                            let mut done = false;
                            let mut offer = |target: Option<Target>| {
                                let bound = target.map(|t| self.bind_target(t));
                                let targets = bound.as_slice();
                                for base in obj.card.cost_options() {
                                    let total = self.target_spell_cost(actor, card, base, targets);
                                    payments(
                                        s.players[actor.index()].mana,
                                        total.colored,
                                        total.generic,
                                        |payment| {
                                            if !done {
                                                done = emit(Action::Cast {
                                                    card,
                                                    target,
                                                    payment,
                                                });
                                            }
                                        },
                                    );
                                    if done {
                                        break;
                                    }
                                }
                                done
                            };
                            if spec.min == 0 && offer(None) {
                                return true;
                            }
                            if spec.max > 0
                                && self.visit_cast_targets(
                                    spec.rule,
                                    actor,
                                    self.object_ref(card),
                                    |t| offer(Some(t)),
                                )
                            {
                                return true;
                            }
                        }
                    }
                    if flashback
                        || exile
                        || target_cost
                        || obj.card.offspring_cost().is_some()
                        || obj.card.teamwork().is_some()
                        || obj.card.kicker_cost().is_some()
                        || obj.card.gift().is_some()
                        || obj.card.collect_evidence_cost().is_some()
                        || obj.card.has_convoke()
                        || improvise::potential(self, obj.card)
                        || self.has_casualty(actor, obj.card)
                        || obj.card.has_bargain()
                        || obj.card.additional_cost().is_some()
                        || obj.card.cost_alternative_guard().is_some()
                        || matches!(obj.card.effect(), Effect::Modal(_))
                    {
                        // Modal spells use the staged path: mode, targets, payment.
                        continue;
                    }
                    let mut done = false;
                    obj.card.reduced_spell_payments(
                        s.players[actor.index()].mana,
                        reduction,
                        |payment| {
                            if done {
                                return;
                            }
                            let plan = match obj.card.effect() {
                                Effect::Spell(plan, _) | Effect::Aura(plan) => plan,
                                _ => TargetPlan::NONE,
                            };
                            let Some(spec) = plan.single() else {
                                if self.cast_targets_available(plan, actor, self.object_ref(card)) {
                                    done = emit(Action::BeginCast { card, payment });
                                }
                                return;
                            };
                            let source = self.object_ref(card);
                            if spec.max > 1 {
                                if self.cast_targets_available(plan, actor, source) {
                                    done = emit(Action::BeginCast { card, payment });
                                }
                            } else {
                                if spec.min == 0
                                    && emit(Action::Cast {
                                        card,
                                        target: None,
                                        payment,
                                    })
                                {
                                    done = true;
                                }
                                if !done && spec.max > 0 {
                                    done = self.visit_cast_targets(
                                        spec.rule,
                                        actor,
                                        source,
                                        |target| {
                                            emit(Action::Cast {
                                                card,
                                                target: Some(target),
                                                payment,
                                            })
                                        },
                                    );
                                }
                            }
                        },
                    );
                    if done {
                        return true;
                    }
                }
            }
            Pending::Attackers => {
                if self.attack_declaration_complete(actor) && emit(Action::FinishAttackers) {
                    return true;
                }
                // Freeze the small defending set once, not one arena scan per
                // candidate attacker. No allocation occurs when there are none.
                let walkers: Vec<_> = s
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(id, o)| {
                        o.controller == actor.other()
                            && o.zone == Zone::Battlefield
                            && self.current_kinds(*id as CardId) & 64 != 0
                    })
                    .map(|(i, _)| self.object_ref(i as CardId))
                    .collect();
                for (i, _) in s.objects.iter().enumerate() {
                    let card = i as CardId;
                    if !self.can_declare_attacker(actor, card)
                        || s.selected_attackers.iter().any(|a| a.card == card)
                    {
                        continue;
                    }
                    if self.can_attack_defender(card, CombatDefender::Player(actor.other()))
                        && emit(Action::Attack { card })
                    {
                        return true;
                    }
                    for &defender in &walkers {
                        if self.can_attack_defender(card, CombatDefender::Planeswalker(defender))
                            && emit(Action::AttackPlaneswalker { card, defender })
                        {
                            return true;
                        }
                    }
                }
            }
            Pending::Blockers => {
                if self.blocks_complete() && emit(Action::FinishBlockers) {
                    return true;
                }
                for (i, _) in s.objects.iter().enumerate() {
                    let blocker = i as CardId;
                    if !self.can_be_blocker(blocker)
                        || s.selected_blocks.iter().any(|&(b, _)| b == blocker)
                    {
                        continue;
                    }
                    for attacker in &s.attackers {
                        if self.can_block_pair(blocker, attacker.card)
                            && self.block_preserves_completion(blocker, attacker.card)
                            && emit(Action::Block {
                                blocker,
                                attacker: attacker.card,
                            })
                        {
                            return true;
                        }
                    }
                }
            }
            Pending::Damage => {
                let range = self.damage_assignment_range().unwrap();
                if range.maximum - range.minimum <= 64 {
                    for amount in range.minimum..=range.maximum {
                        if emit(Action::AssignDamage {
                            target: range.target,
                            amount,
                        }) {
                            return true;
                        }
                    }
                } else {
                    let choice = s.damage_choices.last().unwrap();
                    let lethal = self
                        .lethal_assignment(choice.source, choice.blockers[0])
                        .clamp(range.minimum, range.maximum);
                    let mut prior = None;
                    for amount in [range.minimum, lethal, range.maximum] {
                        if prior != Some(amount)
                            && emit(Action::AssignDamage {
                                target: range.target,
                                amount,
                            })
                        {
                            return true;
                        }
                        prior = Some(amount);
                    }
                }
            }
            Pending::Explore { .. } => {
                let choice = s.resolving.as_ref().unwrap().explore.as_ref().unwrap();
                for graveyard in [false, true] {
                    if emit(Action::ChooseExplore {
                        object: choice.revealed.object,
                        graveyard,
                    }) {
                        return true;
                    }
                }
            }
            Pending::Casualty { .. } => {
                for pay in [false, true] {
                    if self.casualty_intent_fundable(pay) && emit(Action::ChooseCasualty { pay }) {
                        return true;
                    }
                }
            }
            Pending::ManaRollback { .. } => {
                return self.visit_mana_rollback_actions(&mut emit);
            }
            Pending::CastCosts { .. } => {
                return self.visit_cast_cost_clauses(&mut emit);
            }
            Pending::CasualtyPayment { .. } => {
                return self.visit_casualty_payments(&mut emit);
            }
            Pending::Bargain { .. } => {
                let a = s.announcing.as_ref().unwrap();
                if self.bargain_choice_fundable(a, None)
                    && emit(Action::ChooseBargain { sacrifice: None })
                {
                    return true;
                }
                for id in 0..s.objects.len() {
                    let sacrifice = Some(self.object_ref(id as CardId));
                    if self.bargain_choice_fundable(a, sacrifice)
                        && emit(Action::ChooseBargain { sacrifice })
                    {
                        return true;
                    }
                }
            }
            Pending::SourceCounterAmount { .. } => {
                if let Some(domain) = self.source_counter_choice() {
                    if domain.maximum <= 64 {
                        for amount in 0..=domain.maximum {
                            if emit(Action::ChooseSourceCounterAmount { amount }) {
                                return true;
                            }
                        }
                    } else {
                        for amount in [0, domain.maximum / 2, domain.maximum] {
                            if emit(Action::ChooseSourceCounterAmount { amount }) {
                                return true;
                            }
                        }
                    }
                }
            }
            Pending::X { .. } | Pending::ResolutionX { .. } => {
                if let Some(domain) = self.x_choice() {
                    for range in domain.intervals {
                        if range.maximum - range.minimum <= 64 {
                            for value in range.minimum..=range.maximum {
                                if emit(Action::ChooseX { value }) {
                                    return true;
                                }
                            }
                        } else {
                            for value in [
                                range.minimum,
                                range.minimum + (range.maximum - range.minimum) / 2,
                                range.maximum,
                            ] {
                                if emit(Action::ChooseX { value }) {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
            Pending::Kicker { .. } => {
                let a = s.announcing.as_ref().unwrap();
                for pay in [false, true] {
                    if self.kicker_choice_fundable(a, pay) && emit(Action::ChooseKicker { pay }) {
                        return true;
                    }
                }
            }
            Pending::Gift { .. } => {
                let a = s.announcing.as_ref().unwrap();
                for beneficiary in [None, Some(actor.other())] {
                    if self.gift_choice_fundable(a, beneficiary)
                        && emit(Action::ChooseGift { beneficiary })
                    {
                        return true;
                    }
                }
            }
            Pending::Offspring { player: _ } => {
                let a = s.announcing.as_ref().unwrap();
                for pay in [false, true] {
                    if self.offspring_choice_fundable(a, pay)
                        && emit(Action::ChooseOffspring { pay })
                    {
                        return true;
                    }
                }
            }
            Pending::SpellCost { player: _ } | Pending::ActivationCost { player: _ } => {
                let a = s.announcing.as_ref().unwrap();
                for (index, cost) in a.cost_options().enumerate() {
                    if self.announcement_cost_fundable(a, cost)
                        && emit(if a.item.kind == StackKind::Spell {
                            Action::ChooseSpellCost { cost: index as u16 }
                        } else {
                            Action::ChooseActivationCost { cost: index as u16 }
                        })
                    {
                        return true;
                    }
                }
            }
            Pending::Modes { .. } => {
                let item = &s.announcing.as_ref().unwrap().item;
                let modes = item.modes().expect("modal announcement");
                for (index, _) in modes.iter().enumerate() {
                    if self.mode_choice_fundable(s.announcing.as_ref().unwrap(), index)
                        && emit(Action::ChooseMode { mode: index as u16 })
                    {
                        return true;
                    }
                }
            }
            Pending::TargetAmount { .. } => {
                let choice = self.target_amount_choice().unwrap();
                for amount in choice.minimum..=choice.maximum {
                    if emit(Action::AllocateTargetAmount { amount }) {
                        return true;
                    }
                }
            }
            Pending::Retarget { .. } => return self.visit_retarget_actions(&mut emit),
            Pending::Targets { .. } => {
                let announcement = s.announcing.as_ref().unwrap();
                let count = announcement
                    .item
                    .targets
                    .iter()
                    .filter(|bound| bound.group == announcement.group)
                    .count();
                if count >= usize::from(announcement.spec.min)
                    && self.announcement_target_fundable(announcement, None, None, None, None, true)
                    && emit(Action::FinishTargets)
                {
                    return true;
                }
                if count < announcement.spec.max {
                    return self.visit_item_targets(
                        &announcement.item,
                        announcement.spec.rule,
                        |target| {
                            self.target_group_compatible(
                                announcement.spec.rule,
                                announcement.group,
                                &announcement.item.targets,
                                target,
                            ) && !announcement.item.targets.iter().any(|bound| {
                                bound.group == announcement.group && bound.target == target
                            }) && self.announcement_target_fundable(
                                announcement,
                                None,
                                None,
                                None,
                                Some(target),
                                false,
                            ) && emit(Action::ChooseTarget { target })
                        },
                    );
                }
            }
            Pending::ConniveOrder { player } => {
                for conniver in &s
                    .resolving
                    .as_ref()
                    .unwrap()
                    .connive
                    .as_ref()
                    .unwrap()
                    .remaining
                {
                    if conniver.controller == player
                        && emit(Action::ChooseConniver {
                            object: conniver.object,
                        })
                    {
                        return true;
                    }
                }
            }
            Pending::MilledSelection { .. } => {
                if self.can_finish_milled_selection() && emit(Action::FinishMilledSelection) {
                    return true;
                }
                for &object in &s.resolving.as_ref().unwrap().milled {
                    if self.is_milled_selection_candidate(object)
                        && emit(Action::SelectMilled { object })
                    {
                        return true;
                    }
                }
            }
            Pending::HandCardChoice { .. } => return self.visit_hand_card_choices(&mut emit),
            Pending::RevealedDiscard {
                victim,
                excluded_kinds,
                ..
            } => {
                for (id, object) in s.objects.iter().enumerate() {
                    if object.zone == Zone::Hand
                        && object.owner == victim
                        && object.card.kind_mask() & excluded_kinds == 0
                        && emit(Action::ChooseRevealedDiscard {
                            object: self.object_ref(id as CardId),
                        })
                    {
                        return true;
                    }
                }
            }
            Pending::RecoverMilled { player, filter } => {
                if emit(Action::RecoverMilled { object: None }) {
                    return true;
                }
                for &object in &s.resolving.as_ref().unwrap().milled {
                    if self.can_recover_milled(player, filter, object)
                        && emit(Action::RecoverMilled {
                            object: Some(object),
                        })
                    {
                        return true;
                    }
                }
            }
            Pending::EffectSearch { player } => {
                let choice = s.resolving.as_ref().unwrap().search.as_ref().unwrap();
                if choice.destination_pending {
                    for destination in [
                        crate::SearchDestination::Hand,
                        crate::SearchDestination::Graveyard,
                    ] {
                        if emit(Action::ChooseSearchDestination { destination }) {
                            return true;
                        }
                    }
                } else if let Some(selected) = &choice.battlefield {
                    for &card in &choice.selected {
                        if !selected.contains(&card)
                            && emit(Action::ChooseSearchBattlefield { card })
                        {
                            return true;
                        }
                    }
                } else {
                    if choice.selected.len() >= usize::from(choice.minimum)
                        && emit(Action::FinishSearch)
                    {
                        return true;
                    }
                    if choice.selected.len() < usize::from(choice.spec.maximum) {
                        for &card in &s.players[player.index()].library {
                            if choice.spec.filter.matches(s.objects[card as usize].card)
                                && !choice.selected.contains(&card)
                                && (!choice.different_names
                                    || !choice.selected.iter().any(|&id| {
                                        crate::card_names::same_name(
                                            s.objects[id as usize].card.name(),
                                            s.objects[card as usize].card.name(),
                                        )
                                    }))
                                && emit(Action::ChooseSearch { card })
                            {
                                return true;
                            }
                        }
                    }
                }
            }
            Pending::EquipmentAttachment { .. } => {
                return self.visit_return_attachment_actions(emit);
            }
            Pending::EffectLibrarySelection { .. } => {
                let f = s.resolving.as_ref().unwrap().selection.as_ref().unwrap();
                if f.ordering {
                    for r in &f.remaining {
                        if emit(Action::OrderLibraryBottom { card: r.card }) {
                            return true;
                        }
                    }
                } else {
                    if f.selected.len() >= f.minimum && emit(Action::FinishLibrarySelection) {
                        return true;
                    }
                    if f.selected.len() < f.maximum {
                        for r in &f.remaining {
                            if f.eligible.contains(r)
                                && emit(Action::SelectLibrary { card: r.card })
                            {
                                return true;
                            }
                        }
                    }
                }
            }
            Pending::EffectLibrary { .. } => {
                let choice = s.resolving.as_ref().unwrap().library.as_ref().unwrap();
                for &card in &choice.remaining {
                    for destination in [
                        LibraryDestination::Top,
                        if choice.surveil {
                            LibraryDestination::Graveyard
                        } else {
                            LibraryDestination::Bottom
                        },
                    ] {
                        if emit(Action::ArrangeLibrary { card, destination }) {
                            return true;
                        }
                    }
                }
            }
            Pending::OrderTriggers { player } => {
                for item in &s.ordering_triggers {
                    if item.controller == player && emit(Action::PutTrigger { id: item.id }) {
                        return true;
                    }
                }
            }
            Pending::EffectOptional { .. } => {
                if emit(Action::ChooseOptional { accept: true })
                    || emit(Action::ChooseOptional { accept: false })
                {
                    return true;
                }
            }
            Pending::Terminal => {}
        }
        false
    }

    fn apply_legal(&mut self, actor: Player, action: Action) {
        if let Action::PlayGraveyardPermanent {
            permission,
            permanent_type,
            ..
        }
        | Action::AnnounceFreeGraveyardPermanent {
            permission,
            permanent_type,
            ..
        } = &action
        {
            self.consume_graveyard_type(*permission, *permanent_type);
        }
        match action {
            Action::ChooseZoneReplacement { effect } => self.choose_zone_replacement(effect),
            Action::ChooseReplacement { source, ability } => {
                self.choose_replacement(source, ability)
            }
            Action::ChooseDamagePrevention { source, prevention } => {
                self.choose_damage_prevention(source, prevention)
            }
            Action::ChooseDamageEvent { index } => self.choose_damage_event(index),
            Action::ChooseCreatedToken { object } => self.choose_created_token(object),
            Action::ChooseLibraryDestination {
                object,
                destination,
            } => self.choose_library_destination(object, destination),
            Action::ChooseManaValueParity { parity } => self.choose_mana_value_parity(parity),
            Action::ChooseForageRoute { route } => self.choose_forage_route(route),
            Action::ChooseForage { object } => self.choose_forage(object),
            Action::DeclineForage => self.decline_forage(),
            Action::ChooseConniver { object } => {
                self.choose_conniver(object);
                self.run_program();
            }
            Action::PayResolutionMana { payment } => self.finish_resolution_payment(Some(payment)),
            Action::DeclineResolutionMana => self.finish_resolution_payment(None),
            Action::PayResolutionLife => self.finish_resolution_life(true),
            Action::DeclineResolutionLife => self.finish_resolution_life(false),
            Action::ChooseCreatureCounterCost {
                object,
                counter: kind,
            } => {
                if matches!(self.state.pending, Pending::EffectChoice { .. }) {
                    self.pay_resolution_creature_counter(object, kind);
                } else {
                    self.choose_creature_counter_cost(object, kind);
                }
            }
            Action::AnnounceCounterGraveyard { card } => {
                let mut item = self.cast_item(actor, card, [0; 6]);
                item.cast.as_mut().unwrap().method = CastMethod::GraveyardCounterCost;
                self.open_payment_announcement(item, Counters::default(), None);
            }
            Action::AnnounceSelfGrantedAbility {
                source,
                grant_timestamp,
            } => {
                self.announce_self_granted_ability(source, grant_timestamp);
            }
            Action::AnnounceAttachedAbility { grantor, ability } => {
                self.announce_attached_ability(grantor, ability)
            }
            Action::ChooseAdditionalCost { object } => self.choose_additional_cost(object),
            Action::FinishGraveyardSymbolCost => self.finish_graveyard_symbol_cost(),
            Action::ChooseEntryTimestamp { object } => self.choose_entry_timestamp(object),
            Action::ChooseEntryReplacement { source, intrinsic } => {
                self.choose_entry_tap_effect(source, intrinsic)
            }
            Action::AssignLibraryPile { object, pile } => self.assign_library_pile(object, pile),
            Action::ChooseLibraryPile { pile } => self.choose_library_pile(pile),
            Action::KeepPermanentType { object } => self.choose_kept_type(object),
            Action::ChooseEntryValue { value } => self.choose_entry_value(value),
            Action::ChooseEntryLife { pay } => self.choose_entry_life(pay),
            Action::ChooseAuraAttachment { object } => self.choose_aura_attachment(object),
            Action::Keep => {
                self.state.players[actor.index()].kept = true;
                self.next_mulligan(actor);
            }
            Action::Mulligan => {
                // Mulligan decisions for both players precede the actual redraw.
                // Flag the choice using kept=false; next_mulligan executes the round.
                let mulligans = &mut self.state.players[actor.index()].mulligans;
                *mulligans = mulligans
                    .checked_add(1)
                    .capacity("mulligan count exhausted");
                self.state.mulligan_round[actor.index()] = true;
                self.next_mulligan(actor);
            }
            Action::Bottom { card } => {
                self.move_object(card, Zone::Library);
                // move_object appends at the top; bottom ordering is this choice's rule.
                self.state.players[actor.index()].library.pop();
                // Chosen first is put lowest. Subsequent choices go above prior ones.
                let already_bottomed = match self.state.pending {
                    Pending::Bottom { remaining, .. } => {
                        self.state.players[actor.index()].mulligans - remaining
                    }
                    _ => unreachable!(),
                } as usize;
                self.state.players[actor.index()]
                    .library
                    .insert(already_bottomed, card);
                if let Pending::Bottom { remaining, .. } = &mut self.state.pending {
                    *remaining -= 1;
                    if *remaining == 0 {
                        self.next_bottom(actor);
                    }
                }
            }
            Action::Pass => {
                self.state.passes += 1;
                if self.state.passes == 1 {
                    self.state.priority = actor.other();
                } else {
                    self.state.passes = 0;
                    if self.state.stack.is_empty() {
                        self.advance_step();
                    } else {
                        self.resolve_top();
                    }
                }
            }
            Action::RevealMiracle => self.choose_miracle_reveal(true),
            Action::DeclineMiracle => self.choose_miracle_reveal(false),
            Action::CastExiledDuringResolution { object, face } => {
                self.cast_exiled_during_resolution(actor, object, face)
            }
            Action::FinishResolutionCasts => {
                self.finish_paradigm_copy_offer();
                self.state.resolution_free_casts = None;
                self.run_program();
            }
            Action::PlayLand { card }
            | Action::PlayGraveyardPermanent {
                card,
                permanent_type: PermanentType::Land,
                ..
            } => {
                if let Some(free) = self.state.resolution_free_casts.as_mut() {
                    free.remaining.retain(|reference| reference.card != card);
                    self.reveal_hideaway_object(self.object_ref(card));
                }
                if self.state.objects[card as usize].zone == Zone::Library {
                    self.remember_library_top(actor, self.object_ref(card));
                }
                if matches!(
                    self.state.objects[card as usize].card.entry_tapped(),
                    EntryTapped::PayLife(_)
                ) || self.state.objects[card as usize]
                    .card
                    .entry_choice()
                    .is_some()
                    || self.has_entry_counter_replacements(self.state.objects[card as usize].card)
                {
                    // A land play is public even while its pre-entry choice is pending.
                    let reference = self.object_ref(card);
                    if self.state.objects[card as usize].zone == Zone::Hand
                        && !self.state.known_hand.contains(&reference)
                    {
                        self.state.known_hand.push(reference);
                    }
                    let mut movement = self.entry_move(card, Zone::Battlefield, false);
                    movement.controller = actor;
                    movement.public_origin = movement.before.zone == Zone::Library;
                    self.stage_entries(vec![movement], None);
                } else {
                    let mut movement = self.entry_move(card, Zone::Battlefield, false);
                    movement.controller = actor;
                    movement.public_origin = movement.before.zone == Zone::Library;
                    self.stage_entries(vec![movement], None);
                }
                if self.state.land_played {
                    if self.state.players[actor.index()].extra_land_plays_remaining > 0 {
                        self.state.players[actor.index()].extra_land_plays_remaining -= 1;
                    }
                    self.state.continuous_land_plays_used += 1;
                } else {
                    self.state.land_played = true;
                }
                self.state.passes = 0;
            }
            Action::TapGrantedMana {
                card,
                grantor,
                production,
            } => {
                let grant = ability_world::CharacteristicWorld::granted_mana(self, card)
                    .into_iter()
                    .find(|g| {
                        g.source == grantor
                            && g.production
                                .in_world(self, card)
                                .visit(|mana| mana == production)
                    })
                    .expect("legal granted mana ability");
                let quality = self.mana_quality(grant.production, card);
                self.tap_objects(&[card]);
                if grant.sacrifice {
                    self.sacrifice_batch(&[(actor, card)]);
                }
                self.add_quality_mana(actor, production, quality);
                self.state.passes = 0;
            }
            Action::TapMana { card } => {
                self.tap_objects(&[card]);
                let color =
                    ability_world::CharacteristicWorld::intrinsic_mana_color(self, card).unwrap();
                let pool = &mut self.state.players[actor.index()].mana[color];
                *pool = pool
                    .checked_add(1)
                    .capacity("mana representation exhausted");
                self.state.passes = 0;
            }
            Action::AnnounceFree {
                card,
                face,
                flash_surcharge,
            }
            | Action::AnnounceFreeCounterGraveyard {
                card,
                face,
                flash_surcharge,
            }
            | Action::AnnounceFreeGraveyardPermanent {
                card,
                face,
                flash_surcharge,
                ..
            } => {
                let counter_origin = matches!(action, Action::AnnounceFreeCounterGraveyard { .. });
                let mut item = self.cast_item(actor, card, [0; 6]);
                self.set_cast_face(&mut item, face);
                let origin_counter_cost =
                    counter_origin.then(|| item.source_card.graveyard_counter_cost().unwrap());
                item.cast.as_mut().unwrap().method = CastMethod::Free;
                item.cast.as_mut().unwrap().flash_surcharge = flash_surcharge;
                if item.source_card.x_symbols() > 0 {
                    item.x = Some(0);
                }
                self.open_payment_announcement_with_origin_cost(
                    item,
                    Counters::default(),
                    None,
                    origin_counter_cost,
                );
            }
            Action::AnnounceFreeFromPermission {
                card,
                face,
                flash_surcharge,
                permission,
            } => {
                let mut item = self.cast_item(actor, card, [0; 6]);
                self.set_cast_face(&mut item, face);
                item.cast.as_mut().unwrap().method = CastMethod::Free;
                item.cast.as_mut().unwrap().flash_surcharge = flash_surcharge;
                if item.source_card.x_symbols() > 0 {
                    item.x = Some(0);
                }
                self.open_payment_announcement(item, Counters::default(), None);
                self.state
                    .announcing
                    .as_mut()
                    .expect("payment window")
                    .free_hand_permission = Some(Box::new(permission));
            }
            Action::AnnounceGrantedFlashback {
                card,
                flash_surcharge,
            } => {
                let mut item = self.cast_item(actor, card, [0; 6]);
                item.cast.as_mut().unwrap().method = CastMethod::GrantedFlashback;
                item.cast.as_mut().unwrap().flash_surcharge = flash_surcharge;
                self.state.objects[card as usize].flashback = true;
                self.open_payment_announcement(item, Counters::default(), None);
            }
            Action::AnnounceCast { card }
            | Action::AnnounceExile { card }
            | Action::PlayGraveyardPermanent { card, .. } => {
                let item = self.cast_item(actor, card, [0; 6]);
                self.open_payment_announcement(item, Counters::default(), None);
            }
            Action::AnnounceFlashSurcharge { card } => {
                let flashback = self.state.objects[card as usize].zone == Zone::Graveyard;
                let mut item = self.cast_item(actor, card, [0; 6]);
                let cast = item.cast.as_mut().unwrap();
                cast.flash_surcharge = true;
                if flashback {
                    cast.method = CastMethod::Flashback;
                    self.state.objects[card as usize].flashback = true;
                }
                self.open_payment_announcement(item, Counters::default(), None);
            }
            Action::AnnounceBack { card } => {
                let mut item = self.cast_item(actor, card, [0; 6]);
                self.set_cast_face(&mut item, CastFace::Back);
                self.open_payment_announcement(item, Counters::default(), None);
            }
            Action::AnnounceAdventure { card } | Action::AnnounceOmen { card } => {
                let omen = matches!(action, Action::AnnounceOmen { .. });
                let physical = self.state.objects[card as usize].card;
                let definition = if omen {
                    physical.omen_spell()
                } else {
                    physical.adventure_spell()
                }
                .unwrap();
                let mut item = self.cast_item(actor, card, [0; 6]);
                item.cast.as_mut().unwrap().face = if omen {
                    CastFace::Omen
                } else {
                    CastFace::Adventure
                };
                item.cast.as_mut().unwrap().physical_card =
                    Some(self.state.objects[card as usize].card);
                item.source_card = definition;
                item.source_keywords = definition.info().keywords;
                item.source_facts.card = definition;
                item.source_facts.colors = definition.colors();
                item.source_facts.kinds = definition.kind_mask();
                item.source_facts.power = 0;
                self.open_payment_announcement(item, Counters::default(), None);
            }
            Action::AnnounceSneak { card, permission } => {
                if let Some((source, kind)) = permission {
                    self.consume_graveyard_type(source, kind);
                }
                let mut item = self.cast_item(actor, card, [0; 6]);
                item.cast.as_mut().unwrap().method = CastMethod::Sneak;
                self.open_payment_announcement(item, Counters::default(), None);
            }
            Action::AnnounceWarp { card } => {
                let mut item = self.cast_item(actor, card, [0; 6]);
                item.cast.as_mut().unwrap().method = CastMethod::Warp;
                self.open_payment_announcement(item, Counters::default(), None);
            }
            Action::AnnouncePlot { card } => self.begin_plot(actor, card),
            Action::PayPlot { payment } => self.finish_plot(payment),
            Action::AnnouncePlotted { card } => {
                let mut item = self.cast_item(actor, card, [0; 6]);
                item.cast.as_mut().unwrap().method = CastMethod::Plot;
                if item.source_card.x_symbols() > 0 {
                    item.x = Some(0);
                }
                self.open_payment_announcement(item, Counters::default(), None);
            }
            Action::AnnounceFlashback { card } => {
                let mut item = self.cast_item(actor, card, [0; 6]);
                item.cast.as_mut().unwrap().method = CastMethod::Flashback;
                if item.source_card.x_symbols() > 0 {
                    item.x = Some(0);
                }
                self.state.objects[card as usize].flashback = true;
                self.open_payment_announcement(item, Counters::default(), None);
            }
            Action::AnnounceBorrowedAbility {
                card,
                exiled,
                ability,
                counter_payment,
            } => {
                let item = self.borrowed_activation_item(actor, card, exiled, ability);
                self.open_payment_announcement(item, counter_payment, None);
            }
            Action::AnnounceBorrowedMana {
                card,
                exiled,
                ability,
                counter_payment,
                production,
            } => {
                let item = self.borrowed_activation_item(actor, card, exiled, ability);
                self.open_payment_announcement(item, counter_payment, Some(production));
            }
            Action::ActivateBorrowedMana {
                card,
                exiled,
                ability,
                counter_payment,
                payment,
                production,
            } => {
                let mut item = self.borrowed_activation_item(actor, card, exiled, ability);
                let quality = self
                    .mana_quality(
                        item.source_card
                            .activation(ability)
                            .mana_production()
                            .unwrap(),
                        card,
                    )
                    .map(|quality| quality.with_definition(item.source_card));
                self.pay_activation(&mut item, payment, counter_payment);
                self.add_quality_mana(actor, production, quality);
                self.committed_mana_reflexive(&item);
                self.reopen_evidence_after_mana_sacrifice(card);
            }
            Action::AnnounceActivate {
                card,
                ability,
                counter_payment,
            } => {
                let item = self.activation_item(actor, card, ability);
                self.open_payment_announcement(item, counter_payment, None);
            }
            Action::AnnounceMana {
                card,
                ability,
                counter_payment,
                production,
            } => {
                let item = self.activation_item(actor, card, ability);
                self.open_payment_announcement(item, counter_payment, Some(production));
            }
            Action::ChooseEndure { put_counters } => self.choose_endure(put_counters),
            Action::BeginConvoke => self.begin_convoke(),
            Action::BeginImprovise => self.begin_improvise(),
            Action::ChooseImproviser { object } => self.choose_improviser(object),
            Action::ChooseConvoker {
                object,
                contribution,
            } => self.choose_convoker(object, contribution),
            Action::ChooseManaRollback { keep } => self.choose_mana_rollback(keep),
            Action::PayCosts { payment } => {
                let a = self.state.announcing.as_mut().unwrap();
                a.payment = Some(payment);
                if let Some(c) = &mut a.cast_costs
                    && let Some(index) = c
                        .clauses
                        .iter()
                        .position(|r| r.clause == CastCostClause::Mana && !r.paid)
                {
                    c.active = Some(index as u16);
                }
                self.finish_announcement();
            }
            Action::Cast {
                card,
                target,
                payment,
            } => {
                let mut item = self.cast_item(actor, card, payment);
                if let Some(target) = target {
                    let bound = self.bind_target(target);
                    self.target_becomes_bound(&item, bound);
                    item.targets.push(bound);
                }
                self.push_stack_item(item);
                self.state.passes = 0;
            }
            Action::Activate {
                card,
                ability,
                target,
                payment,
                counter_payment,
            } => {
                let mut item = self.activation_item(actor, card, ability);
                if let Some(target) = target {
                    let bound = self.bind_target(target);
                    self.target_becomes_bound(&item, bound);
                    item.targets.push(bound);
                }
                item.allocate_single_target();
                self.pay_activation(&mut item, payment, counter_payment);
                self.push_stack_item(item);
                self.state.passes = 0;
            }
            Action::ActivateMana {
                card,
                ability,
                payment,
                counter_payment,
                production,
            } => {
                let mut item = self.activation_item(actor, card, ability);
                let definition = self.state.objects[card as usize]
                    .card
                    .activation(ability)
                    .mana_production()
                    .unwrap();
                let quality = self.mana_quality(definition, card);
                self.pay_activation(&mut item, payment, counter_payment);
                self.add_quality_mana(actor, production, quality);
                self.committed_mana_reflexive(&item);
                self.reopen_evidence_after_mana_sacrifice(card);
            }
            Action::ChooseManaLotSpend { .. } => unreachable!("allocation handled transactionally"),
            Action::ChooseMana { production } => {
                let quality = self
                    .state
                    .mana_lots
                    .as_mut()
                    .and_then(|state| state.effect_quality.take());
                self.add_quality_mana(actor, production, quality);
                self.run_program();
            }
            Action::BeginActivate {
                card,
                ability,
                payment,
                counter_payment,
            } => {
                let item = self.activation_item(actor, card, ability);
                if self.start_announcement(item, false) {
                    let announcement = self.state.announcing.as_mut().unwrap();
                    announcement.payment = Some(payment);
                    announcement.counter_payment = counter_payment;
                } else {
                    let mut item = self.state.stack.last().unwrap().clone();
                    self.pay_activation(&mut item, payment, counter_payment);
                }
            }
            Action::BeginCast { card, payment } => {
                let item = self.cast_item(actor, card, [0; 6]);
                if self.start_announcement(item, false) {
                    self.state.announcing.as_mut().unwrap().payment = Some(payment);
                } else {
                    let item = self.state.stack.last().unwrap().clone();
                    let outcome = self.spend_mana_for(actor, payment, self.purpose_for_item(&item));
                    if outcome.haste {
                        self.grant_stack_keywords(item.object, Keywords::HASTE);
                    }
                    self.state.stack.last_mut().unwrap().mana_copy_sources = outcome.copies;
                    self.state
                        .stack
                        .last_mut()
                        .unwrap()
                        .cast
                        .as_mut()
                        .unwrap()
                        .payment = payment;
                    self.record_cast(
                        actor,
                        self.state.objects[card as usize].card,
                        Some(self.state.stack.len() - 1),
                    );
                }
            }
            Action::KeepRole { object } => self.keep_role(object),
            Action::KeepLegend { object } => self.keep_legend(object),
            Action::ChooseCollectEvidence { collect } => {
                self.state
                    .announcing
                    .as_mut()
                    .unwrap()
                    .item
                    .cast
                    .as_mut()
                    .unwrap()
                    .evidence = Some(collect);
                self.continue_announcement_choices(false);
            }
            Action::ChooseEvidence { object } => self.choose_evidence(object),
            Action::FinishEvidence => self.finish_evidence(),
            Action::DeclineEvidence => self.decline_evidence(),
            Action::ChooseExplore { graveyard, .. } => self.finish_explore(graveyard),
            Action::ChooseCasualty { pay } => self.choose_casualty(pay),
            Action::ChooseCastCost { clause } => self.choose_cast_cost(clause),
            Action::PayCasualty { object } => self.pay_casualty_clause(object),
            Action::ChooseBargain { sacrifice } => {
                let a = self.state.announcing.as_mut().unwrap();
                a.item.cast.as_mut().unwrap().bargain = Some(sacrifice.is_some());
                a.additional = sacrifice;
                self.continue_announcement_choices(false);
            }
            Action::ChooseMobilizeDefender { defender } => self.choose_mobilize_defender(defender),
            Action::ChooseTeamwork { pay } => self.choose_teamwork(pay),
            Action::ChooseTeammate { object } => self.choose_teammate(object),
            Action::FinishTeamwork => self.finish_teamwork(),
            Action::ChooseSaddler { object } => self
                .state
                .saddle_payment
                .as_mut()
                .unwrap()
                .selected
                .push(object),
            Action::FinishSaddle => self.finish_saddle_payment(),
            Action::ChooseCrewer { object } => self.choose_crewer(object),
            Action::FinishCrew => self.finish_crew_payment(),
            Action::ChooseSourceCounterAmount { amount } => {
                self.choose_source_counter_amount(amount)
            }
            Action::ChooseX { value }
                if matches!(self.state.pending, Pending::ResolutionX { .. }) =>
            {
                let item = {
                    let r = self.state.resolving.as_mut().unwrap();
                    r.item.x = Some(value);
                    r.item.clone()
                };
                let resolving = self.state.resolving.as_ref().unwrap();
                if value == 0
                    && matches!(
                        resolving.program.get(resolving.pc.saturating_sub(1)),
                        Some(Instruction::PayXReflexiveLimited { .. })
                    )
                {
                    self.run_program();
                } else {
                    self.start_optional_resolution_payment(
                        &item,
                        ManaCost {
                            colored: [0; 6],
                            generic: value,
                        },
                        0,
                    );
                }
            }
            Action::ChooseX { value } => {
                self.state.announcing.as_mut().unwrap().item.x = Some(value);
                let a = self.state.announcing.as_mut().unwrap();
                if a.item.kind == StackKind::Spell && a.item.source_card.life_x_cost() {
                    a.add_cast_cost_clause(CastCostClause::Life(value));
                }
                if self
                    .state
                    .announcing
                    .as_ref()
                    .unwrap()
                    .item
                    .source_card
                    .targets_up_to_x()
                {
                    let plan = self.target_spec(&self.state.announcing.as_ref().unwrap().item);
                    self.state.announcing.as_mut().unwrap().plan = plan;
                }
                self.continue_announcement_choices(false);
            }
            Action::ChooseKicker { pay } => {
                self.state
                    .announcing
                    .as_mut()
                    .unwrap()
                    .item
                    .cast
                    .as_mut()
                    .unwrap()
                    .kicker = Some(pay);
                if let Some(plan) = self
                    .state
                    .announcing
                    .as_ref()
                    .unwrap()
                    .item
                    .source_card
                    .kicked_targets()
                    .filter(|_| pay)
                {
                    self.state.announcing.as_mut().unwrap().plan = plan;
                }
                self.continue_announcement_choices(false);
            }
            Action::ChooseGift { beneficiary } => self.choose_gift(beneficiary),
            Action::ChooseOffspring { pay } => {
                self.state
                    .announcing
                    .as_mut()
                    .unwrap()
                    .item
                    .cast
                    .as_mut()
                    .unwrap()
                    .offspring = Some(pay);
                self.continue_announcement_choices(false);
            }
            Action::ChooseSpellCost { cost } | Action::ChooseActivationCost { cost } => {
                self.choose_mana_cost(cost)
            }
            Action::ChooseMode { mode } => self.choose_mode(mode),
            Action::FinishStackTargets => self.finish_stack_targets(),
            Action::AllocateTargetAmount { amount } => self.choose_target_amount(amount),
            Action::SetStackTarget { index, target } => self.set_stack_target(index, target),
            Action::RestoreStackTarget { index } => self.restore_stack_target(index),
            Action::ChooseTarget { target } => {
                let mut bound = self.bind_target(target);
                let item = self.state.announcing.as_ref().unwrap().item.clone();
                self.target_becomes_bound(&item, bound);
                let announcement = self.state.announcing.as_mut().unwrap();
                bound.group = announcement.group;
                announcement.item.targets.push(bound);
                if announcement
                    .item
                    .targets
                    .iter()
                    .filter(|bound| bound.group == announcement.group)
                    .count()
                    == announcement.spec.max
                {
                    self.finish_target_group();
                }
            }
            Action::FinishTargets => self.finish_target_group(),
            Action::Attack { card } => self.state.selected_attackers.push(Attacker {
                card,
                defender: Some(CombatDefender::Player(actor.other())),
                blocked: false,
            }),
            Action::AttackPlaneswalker { card, defender } => {
                self.state.selected_attackers.push(Attacker {
                    card,
                    defender: Some(CombatDefender::Planeswalker(defender)),
                    blocked: false,
                })
            }
            Action::FinishAttackers => {
                let first_attacks: Vec<_> = self
                    .state
                    .selected_attackers
                    .iter()
                    .filter(|a| {
                        !self
                            .state
                            .attacked_this_turn
                            .contains(&self.object_ref(a.card))
                    })
                    .map(|a| a.card)
                    .collect();
                let declared_count = self.state.selected_attackers.len();
                self.commit_attackers_with(actor, |game, card| {
                    game.attack_triggers_for_declaration(
                        &game.object_view(card),
                        declared_count,
                        first_attacks.contains(&card),
                    );
                });
                self.give_priority();
            }
            Action::Block { blocker, attacker } => {
                self.state.selected_blocks.push((blocker, attacker))
            }
            Action::FinishBlockers => {
                self.state
                    .blocking
                    .extend(self.state.selected_blocks.iter().map(|&(b, _)| b));
                self.state.blocks.append(&mut self.state.selected_blocks);
                self.attachment_block_triggers(actor);
                for attacker in &mut self.state.attackers {
                    attacker.blocked = self.state.blocks.iter().any(|&(_, a)| a == attacker.card);
                }
                self.give_priority();
            }
            Action::AssignDamage { target, amount } => {
                let Target::Card(blocker) = target else {
                    unreachable!()
                };
                let source = self.state.damage_choices.last().unwrap().source;
                let lethal = amount >= self.lethal_assignment(source, blocker);
                self.queue_hit(source, target, amount, true);
                let choice = self.state.damage_choices.last_mut().unwrap();
                choice.remaining -= amount;
                choice.blockers.remove(0);
                choice.prior_blockers_lethal &= lethal;
                if choice.blockers.is_empty() {
                    let choice = self.state.damage_choices.pop().unwrap();
                    if let Some(target) = self.combat_defender_target(choice.defender) {
                        self.queue_hit(source, target, choice.remaining, true);
                    }
                }
                self.advance_damage_choices();
            }
            Action::ChooseToken { token } => {
                self.create_tokens(actor, token, 1, false);
                self.run_program();
            }
            Action::PutHandCreature { object }
            | Action::PutHandPermanent { object }
            | Action::PutHandLand { object }
            | Action::SelectLand { object } => self.choose_effect_object(actor, object),
            Action::FinishLandSelection => {
                self.finish_land_selection();
                self.run_program();
            }
            Action::DeclineHandEntry => self.decline_effect_payment(),
            Action::ChooseHandDefender { defender } => self.choose_hand_defender(defender),
            Action::Sacrifice { object } => self.choose_effect_object(actor, object),
            Action::DeclineEffectPayment => self.decline_effect_payment(),
            Action::Discard { card } => {
                if matches!(self.state.pending, Pending::ResolutionMana { .. }) {
                    self.finish_ward_discard(card);
                } else if matches!(self.state.pending, Pending::EffectChoice { .. }) {
                    self.choose_effect_object(actor, self.object_ref(card));
                } else if matches!(self.state.pending, Pending::EffectDiscard { .. }) {
                    let resolution = self.state.resolving.as_mut().unwrap();
                    resolution.discards[actor.index()] -= 1;
                    resolution.selected_discards[actor.index()].push(card);
                    self.run_program();
                } else {
                    self.state.cleanup_discards.push(card);
                    if !self.above_hand_limit(actor, self.state.cleanup_discards.len()) {
                        let discarded = std::mem::take(&mut self.state.cleanup_discards);
                        self.discard_batch(&discarded, None);
                        self.finish_cleanup();
                    }
                }
            }
            Action::PutTrigger { id } => {
                let index = self
                    .state
                    .ordering_triggers
                    .iter()
                    .position(|item| item.id == id)
                    .unwrap();
                let item = self.state.ordering_triggers.remove(index);
                if !self.start_announcement(item, true) {
                    self.continue_trigger_order();
                }
            }
            Action::SelectMilled { object } => self.select_milled(object),
            Action::FinishMilledSelection => self.finish_milled_selection(),
            Action::ChooseHandCard { object } => self.choose_hand_card(object),
            Action::ChooseRevealedDiscard { object } => {
                let cause = self.state.resolving.as_ref().unwrap().item.controller;
                self.discard_batch_caused(&[object.card], None, Some(cause));
                self.run_program();
            }
            Action::RecoverMilled { object } => self.recover_milled(object),
            Action::ChooseSearchZones { library, graveyard } => {
                self.choose_search_zones(library, graveyard)
            }
            Action::ChooseSearch { card } => {
                if self.has_zone_search() {
                    self.choose_zone_search_card(card)
                } else {
                    self.choose_search(card)
                }
            }
            Action::FinishSearch => {
                if self.has_zone_search() {
                    self.finish_zone_search()
                } else {
                    self.finish_search(actor)
                }
            }
            Action::ChooseSearchBattlefield { card } => self.choose_search_battlefield(actor, card),
            Action::ChooseSearchDestination { destination } => {
                self.choose_search_destination(actor, destination)
            }
            Action::SelectLibrary { card } => self.choose_library_selection(card),
            Action::ChooseEquipmentAttachment { equipment } => {
                self.choose_equipment_attachment(equipment)
            }
            Action::FinishLibrarySelection => self.finish_library_selection(actor),
            Action::OrderLibraryBottom { card } => self.order_library_bottom(actor, card),
            Action::ArrangeLibrary { card, destination } => {
                self.arrange_library(actor, card, destination);
            }
            Action::BeginOnBattlefield { object } => {
                let Pending::OpeningHand { eligible, .. } = &mut self.state.pending else {
                    unreachable!()
                };
                eligible[actor.index()].retain(|&candidate| candidate != object);
                let movement = self.entry_move(object.card, Zone::Battlefield, false);
                self.stage_entries(vec![movement], None);
            }
            Action::FinishOpeningHand => {
                if actor == self.state.starting {
                    let Pending::OpeningHand { player, .. } = &mut self.state.pending else {
                        unreachable!()
                    };
                    *player = actor.other();
                } else {
                    self.begin_turn();
                }
            }
            Action::ChooseOptional { accept }
                if matches!(self.state.pending, Pending::OptionalSearch(_)) =>
            {
                self.choose_optional_search(actor, accept);
            }
            Action::ChooseOptional { accept } => {
                if self.complete_library_entry_transform(accept) {
                    self.run_program();
                    return;
                }
                let Pending::EffectOptional { skip_to, .. } = self.state.pending else {
                    unreachable!()
                };
                if !accept {
                    self.state.resolving.as_mut().unwrap().pc = skip_to;
                } else {
                    self.complete_optional_tap_reflexive();
                    self.complete_optional_discard_exile_play();
                }
                self.complete_optional_source_counter_cast(accept);
                self.run_program();
            }
            Action::Concede => unreachable!(),
        }
    }

    fn next_mulligan(&mut self, actor: Player) {
        let start = self.state.starting;
        if actor == start && !self.state.players[start.other().index()].kept {
            self.state.pending = Pending::Mulligan(start.other());
            return;
        }
        // Everyone who chose to mulligan shuffles and draws simultaneously.
        for p in [start, start.other()] {
            if self.state.mulligan_round[p.index()] {
                let hand: Vec<_> = self
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(_, obj)| obj.owner == p && obj.zone == Zone::Hand)
                    .map(|(i, _)| i as CardId)
                    .collect();
                for id in hand {
                    self.move_object(id, Zone::Library);
                }
                self.forget_library_top(p);
                self.state
                    .rng
                    .shuffle(&mut self.state.players[p.index()].library);
                for _ in 0..7 {
                    self.draw(p);
                }
            }
        }
        for p in [start, start.other()] {
            if self.state.mulligan_round[p.index()] {
                self.state.pending = Pending::Bottom {
                    player: p,
                    remaining: self.state.players[p.index()].mulligans,
                };
                return;
            }
        }
        self.begin_mulligan_round();
    }

    fn next_bottom(&mut self, actor: Player) {
        self.state.mulligan_round[actor.index()] = false;
        let other = self.state.starting.other();
        if actor == self.state.starting && self.state.mulligan_round[other.index()] {
            self.state.pending = Pending::Bottom {
                player: other,
                remaining: self.state.players[other.index()].mulligans,
            };
        } else {
            self.begin_mulligan_round();
        }
    }

    fn begin_mulligan_round(&mut self) {
        for p in [self.state.starting, self.state.starting.other()] {
            if self.state.players[p.index()].mulligans == 7 {
                self.state.players[p.index()].kept = true;
            }
            if !self.state.players[p.index()].kept {
                self.state.pending = Pending::Mulligan(p);
                return;
            }
        }
        let eligible = std::array::from_fn(|index| {
            self.state
                .objects
                .iter()
                .enumerate()
                .filter(|(_, object)| {
                    object.owner.index() == index
                        && object.zone == Zone::Hand
                        && object.card.opening_hand_battlefield()
                })
                .map(|(id, _)| self.object_ref(id as CardId))
                .collect()
        });
        self.state.pending = Pending::OpeningHand {
            player: self.state.starting,
            eligible: Box::new(eligible),
        };
    }

    fn hand_count(&self, p: Player) -> usize {
        self.state
            .objects
            .iter()
            .filter(|o| o.owner == p && o.zone == Zone::Hand && !o.is_token())
            .count()
    }

    fn draw(&mut self, p: Player) {
        match self.state.players[p.index()].library.last().copied() {
            Some(id) => {
                self.move_object(id, Zone::Hand);
                self.card_drawn(p);
                self.offer_miracle_reveal(p, id);
            }
            None => self.state.players[p.index()].attempted_empty_draw = true,
        }
    }

    fn on_battlefield(&self, id: CardId) -> bool {
        self.state
            .objects
            .get(id as usize)
            .is_some_and(|o| o.zone == Zone::Battlefield)
    }

    fn object_ref(&self, card: CardId) -> ObjectRef {
        ObjectRef {
            card,
            generation: self.state.objects[card as usize].generation,
        }
    }

    fn object_view(&self, id: CardId) -> ObjectView {
        self.characteristic_query().object_view(id)
    }

    /// All real zone changes go through here. The library's default insertion is
    /// its top; effects that specify ordering reposition after the transition.
    /// The returned event is owned so later trigger processing can retain LKI.
    fn move_object(&mut self, id: CardId, destination: Zone) -> ZoneChange {
        let sources = if self.state.objects[id as usize].zone == Zone::Battlefield
            && destination == Zone::Graveyard
        {
            self.death_trigger_sources()
        } else {
            Vec::new()
        };
        let event = self.move_object_raw(id, destination);
        self.finish_zone_events(std::slice::from_ref(&event), &sources);
        event
    }

    /// Sacrifices are an explicit action, including costs paid for mana abilities.
    /// Freeze every observer and departing object before the simultaneous batch.
    fn sacrifice_batch(&mut self, sacrifices: &[(Player, CardId)]) {
        for (index, &(actor, id)) in sacrifices.iter().enumerate() {
            let object = &self.state.objects[id as usize];
            assert_eq!(
                object.zone,
                Zone::Battlefield,
                "sacrifice requires a permanent"
            );
            assert_eq!(object.controller, actor, "sacrifice requires control");
            debug_assert!(
                !sacrifices[..index].iter().any(|&(_, prior)| prior == id),
                "duplicate sacrifice"
            );
        }
        let sources = self.death_trigger_sources();
        self.food_sacrifice_triggers(sacrifices, &sources);
        // Most sacrifice costs move one object. Avoid batch-vector allocations
        // in this hot path while retaining the same frozen observer boundary.
        if let [(actor, id)] = sacrifices {
            let mut event = self.move_object_raw(*id, Zone::Graveyard);
            event.sacrificed_by = Some(*actor);
            self.record_own_sacrifices(std::slice::from_ref(&event));
            self.finish_zone_events(std::slice::from_ref(&event), &sources);
            return;
        }
        let ids: Vec<_> = sacrifices.iter().map(|&(_, id)| id).collect();
        let mut events = self.move_objects_raw(&ids, Zone::Graveyard);
        for (event, &(actor, _)) in events.iter_mut().zip(sacrifices) {
            event.sacrificed_by = Some(actor);
        }
        self.record_own_sacrifices(&events);
        self.finish_zone_events(&events, &sources);
    }

    /// Capture every pre-event characteristic before any simultaneous departure.
    fn move_objects_raw(&mut self, ids: &[CardId], destination: Zone) -> Vec<ZoneChange> {
        let returned_hand_observers =
            if destination == Zone::Hand && ids.iter().any(|&id| self.on_battlefield(id)) {
                self.freeze_returned_hand_observers()
            } else {
                None
            };
        let graveyard_observers = if ids
            .iter()
            .any(|&id| self.state.objects[id as usize].zone == Zone::Graveyard)
        {
            self.freeze_graveyard_observers()
        } else {
            None
        };
        if destination == Zone::Battlefield {
            let count = ids
                .iter()
                .filter(|&&id| {
                    let o = &self.state.objects[id as usize];
                    o.zone != Zone::Battlefield
                        && !o.is_token()
                        && !o.card.is_emblem()
                        && (o.card.base_stat_setter().is_some()
                            || !o.card.live_static_effects().is_empty()
                            || o.card.role().is_some()
                            || o.card.has_layer6_provider())
                })
                .count();
            assert!(count <= 1, "simultaneous timestamp entry must be staged");
            if ids.iter().any(|&id| {
                self.state.objects[id as usize]
                    .card
                    .hand_size_setting()
                    .flatten()
                    .is_some()
            }) {
                assert!(
                    ids.iter()
                        .filter(
                            |&&id| self.state.objects[id as usize].zone != Zone::Battlefield
                                && self.state.objects[id as usize]
                                    .card
                                    .hand_size_setting()
                                    .is_some()
                        )
                        .count()
                        <= 1,
                    "simultaneous hand timestamp entry must be staged"
                );
            }
        }
        let before: Vec<_> = ids
            .iter()
            .map(|&id| {
                assert!(
                    destination != Zone::Battlefield
                        || !matches!(
                            self.state.objects[id as usize].card.entry_tapped(),
                            EntryTapped::PayLife(_)
                        ),
                    "choice-bearing entry must be staged"
                );
                let view = self.object_view(id);
                let counters = self.pre_entry_counters(&view, destination);
                (
                    self.freeze_zone_rules(&view, destination, None),
                    view,
                    destination == Zone::Battlefield && self.enters_tapped(id),
                    counters,
                )
            })
            .collect();
        self.capture_event_object_facts(before.iter().map(|(_, view, _, _)| view));
        self.capture_source_quantities(before.iter().map(|(_, view, _, _)| view));
        // Replacement decisions precede the entire simultaneous commit.
        let planned: Vec<_> = before
            .into_iter()
            .map(|(rules, before, tapped, counters)| {
                let plan = self.plan_zone_move(&before, destination, rules);
                (before, plan, tapped, counters)
            })
            .collect();
        let events = planned
            .into_iter()
            .map(|(before, plan, tapped, counters)| {
                self.commit_planned_zone_move(
                    before,
                    plan,
                    tapped,
                    counters,
                    None,
                    ZoneObservers {
                        graveyard: graveyard_observers.clone(),
                        returned_hand: returned_hand_observers.clone(),
                    },
                )
            })
            .collect();
        self.refresh_controllers();
        events
    }

    pub(super) fn opponent_entry_requires_tapped(&self, card: Card, controller: Player) -> bool {
        self.opponent_entry_kinds_require_tapped(card.kind_mask(), controller)
    }
    pub(super) fn opponent_entry_kinds_require_tapped(
        &self,
        kinds: u16,
        controller: Player,
    ) -> bool {
        self.state.static_sources.iter().any(|&id| {
            let source = &self.state.objects[id as usize];
            source.zone == Zone::Battlefield
                && source.controller != controller
                && self.printed_abilities_available(id)
                && source.card.opponent_entry_tapped() & kinds != 0
        })
    }

    fn enters_tapped(&self, id: CardId) -> bool {
        let object = &self.state.objects[id as usize];
        self.enters_tapped_for(id, object.controller)
    }

    fn enters_tapped_for(&self, id: CardId, controller: Player) -> bool {
        self.opponent_entry_kinds_require_tapped(self.current_kinds(id), controller)
            || self.self_entry_requires_tapped(id, controller)
    }

    fn self_entry_requires_tapped(&self, id: CardId, controller: Player) -> bool {
        let object = &self.state.objects[id as usize];
        match object.card.entry_tapped() {
            EntryTapped::Never | EntryTapped::PayLife(_) => false,
            EntryTapped::UnlessPlainsOrIsland => {
                !self.state.objects.iter().enumerate().any(|(id, o)| {
                    o.zone == Zone::Battlefield
                        && o.controller == controller
                        && (self.has_subtype(id as CardId, "Plains")
                            || self.has_subtype(id as CardId, "Island"))
                })
            }
            EntryTapped::UnlessPlaneswalker => {
                !self.state.objects.iter().enumerate().any(|(id, o)| {
                    o.zone == Zone::Battlefield
                        && o.controller == controller
                        && self.current_kinds(id as CardId) & 64 != 0
                })
            }
            EntryTapped::Always | EntryTapped::ChosenXAtMost(_) => true,
            EntryTapped::OpponentsTurn => controller != self.state.active,
            EntryTapped::UnlessLowLife => self.state.players.iter().all(|p| p.life > 13),
            EntryTapped::UnlessBasic => !self.state.objects.iter().enumerate().any(|(id, o)| {
                o.zone == Zone::Battlefield
                    && o.controller == controller
                    && self.current_kinds(id as CardId) & 2 != 0
                    && o.card.is_basic_land()
            }),
            EntryTapped::UnlessAtMostTwo | EntryTapped::UnlessAtLeastTwo => {
                let lands = self
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(other, o)| {
                        *other != id as usize
                            && o.zone == Zone::Battlefield
                            && o.controller == controller
                            && self.current_kinds(*other as CardId) & 2 != 0
                    })
                    .count();
                match object.card.entry_tapped() {
                    EntryTapped::UnlessAtMostTwo => lands > 2,
                    _ => lands < 2,
                }
            }
        }
    }

    fn move_object_raw(&mut self, id: CardId, destination: Zone) -> ZoneChange {
        let tapped = destination == Zone::Battlefield && self.enters_tapped(id);
        self.move_object_raw_with_entry(id, destination, tapped)
    }

    fn move_object_raw_with_entry(
        &mut self,
        id: CardId,
        destination: Zone,
        tapped: bool,
    ) -> ZoneChange {
        assert!(
            destination != Zone::Battlefield
                || !matches!(
                    self.state.objects[id as usize].card.entry_tapped(),
                    EntryTapped::PayLife(_)
                ),
            "choice-bearing entry must be staged"
        );
        let before = self.object_view(id);
        let event = self.transition_object(before, destination, tapped);
        self.refresh_controllers();
        event
    }

    fn transition_object(
        &mut self,
        before: ObjectView,
        destination: Zone,
        tapped: bool,
    ) -> ZoneChange {
        let counters = self.pre_entry_counters(&before, destination);
        self.transition_object_with_counters(before, destination, tapped, counters)
    }

    fn transition_object_with_counters(
        &mut self,
        before: ObjectView,
        destination: Zone,
        tapped: bool,
        counters: Counters,
    ) -> ZoneChange {
        self.transition_object_under_control(before, destination, tapped, counters, None)
    }

    fn transition_object_under_control(
        &mut self,
        before: ObjectView,
        destination: Zone,
        tapped: bool,
        counters: Counters,
        entry_controller: Option<Player>,
    ) -> ZoneChange {
        self.capture_event_object_facts(std::iter::once(&before));
        self.capture_source_quantities(std::iter::once(&before));
        let rules = self.freeze_zone_rules(&before, destination, None);
        let plan = self.plan_zone_move(&before, destination, rules);
        let returned_hand_observers =
            if before.zone == Zone::Battlefield && destination == Zone::Hand {
                self.freeze_returned_hand_observers()
            } else {
                None
            };
        let graveyard_observers = if before.zone == Zone::Graveyard {
            self.freeze_graveyard_observers()
        } else {
            None
        };
        self.commit_planned_zone_move(
            before,
            plan,
            tapped,
            counters,
            entry_controller,
            ZoneObservers {
                graveyard: graveyard_observers,
                returned_hand: returned_hand_observers,
            },
        )
    }

    fn commit_planned_zone_move(
        &mut self,
        before: ObjectView,
        plan: zone_replacements::Plan,
        tapped: bool,
        counters: Counters,
        entry_controller: Option<Player>,
        observers: ZoneObservers,
    ) -> ZoneChange {
        let ZoneObservers {
            graveyard: graveyard_observers,
            returned_hand: returned_hand_observers,
        } = observers;
        let id = before.id;
        self.capture_mana_copy_source_departure(&before);
        if before.zone == Zone::Stack {
            self.capture_stack_copy_departure(id);
        }
        if before.zone == Zone::Stack
            && plan.destination != Zone::Battlefield
            && self.state.objects[id as usize].is_spell_copy()
        {
            self.state.stack.retain(|item| {
                !(item.kind == StackKind::Spell
                    && item.card == id
                    && item.object == before.reference)
            });
            self.clear_copy_recipe(before.reference);
            let object = &mut self.state.objects[id as usize];
            object.zone = Zone::Outside;
            object.generation = object
                .generation
                .checked_add(1)
                .capacity("object generation exhausted");
            self.state.free_tokens.push(id);
            return ZoneChange {
                cast_event: None,
                entry_counter_player: None,
                graveyard_observers: None,
                returned_hand_observers: None,
                was_cast: false,
                gift: None,
                after: self.object_ref(id),
                destination: Zone::Outside,
                before,
                moved: false,
                sacrificed_by: None,
                offspring_paid: false,
                kicked: false,
                bargained: false,
                x: 0,
                evidence_collected: false,
            };
        }
        let destination = plan.destination;
        if before.zone == Zone::Graveyard
            && destination != Zone::Graveyard
            && !before.is_token()
            && !before.card.is_emblem()
            && !self.state.objects[id as usize].is_spell_copy()
        {
            self.state.players[before.owner.index()].card_left_graveyard_this_turn = true;
        }
        // Outside is an internal staging zone for newly allocated tokens. A
        // ceased token is on the free list and cannot move again (CR111.8).
        let fresh_token = before.zone == Zone::Outside && !self.state.free_tokens.contains(&id);
        if (self.state.objects[id as usize].copy_kind == CopyKind::Spell
            && before.zone != Zone::Stack)
            || (self.state.objects[id as usize].copy_kind == CopyKind::Card
                && before.zone == Zone::Outside)
            || before.card.is_emblem()
            || (before.is_token() && before.zone != Zone::Battlefield && !fresh_token)
        {
            return ZoneChange {
                cast_event: None,
                entry_counter_player: None,
                graveyard_observers: None,
                returned_hand_observers: None,
                was_cast: false,
                gift: None,
                after: before.reference,
                destination: before.zone,
                before,
                moved: false,
                sacrificed_by: None,
                offspring_paid: false,
                kicked: false,
                bargained: false,
                x: 0,
                evidence_collected: false,
            };
        }
        if before.zone == Zone::Library && destination == Zone::Library && plan.shuffle {
            self.reveal_and_shuffle_zone_card(&before);
            return ZoneChange {
                cast_event: None,
                entry_counter_player: None,
                graveyard_observers: None,
                returned_hand_observers: None,
                was_cast: false,
                gift: None,
                after: before.reference,
                destination,
                before,
                moved: false,
                sacrificed_by: None,
                offspring_paid: false,
                kicked: false,
                bargained: false,
                x: 0,
                evidence_collected: false,
            };
        }
        if destination == Zone::Exile
            && !before.is_token()
            && !before.card.is_emblem()
            && !self.state.objects[id as usize].is_spell_copy()
        {
            self.state.cards_exiled_this_turn = true;
        }
        // CR400.8: re-exiling an exiled card creates a new incarnation even
        // though its zone stays exile. Clear exact links and permissions below.
        assert!(
            before.zone != destination || destination == Zone::Exile,
            "same-zone moves are ordering operations except re-exile"
        );
        if before.zone == Zone::Graveyard {
            self.state.step_sources.retain(|&source| source != id);
            self.state.damage_sources.retain(|&source| source != id);
        }
        self.forget_borrowed_source(before.reference);
        self.forget_power_up_object(before.reference);
        if let Some(state) = &mut self.state.roles {
            state
                .harnessed
                .retain(|reference| *reference != before.reference);
        }
        self.forget_face_change(before.reference);
        if let Some(state) = &mut self.state.roles {
            state.keyword_effects.retain(|row| {
                row.recipient != before.reference
                    || !matches!(row.origin, KeywordEffectOrigin::Counter { .. })
            });
        }
        self.forget_stack_graveyard_exile(before.reference);
        self.forget_cast_entry_counters(before.reference);
        self.capture_source_counter_departure(&before);
        self.clear_entry_value(before.reference);
        if before.zone == Zone::Battlefield {
            self.forget_prepared(before.reference);
            self.clear_suspected_reference(before.reference);
            self.forget_graveyard_type_source(before.reference);
        }
        if before.zone == Zone::Battlefield
            && destination == Zone::Graveyard
            && before.is_creature()
        {
            self.state.players[before.controller.index()].controlled_creature_died_this_turn = true;
            self.state.creatures_died_this_turn = self
                .state
                .creatures_died_this_turn
                .checked_add(1)
                .capacity("creature death history exhausted");
        }
        if before.zone == Zone::Battlefield {
            self.capture_stat_subject_departure(&before);
            self.forget_death_exile_replacements(before.reference);
            self.forget_vehicle_animation(before.reference);
            self.state.saddled.retain(|r| *r != before.reference);
        }
        self.forget_adventure(before.reference);
        let generation = before
            .reference
            .generation
            .checked_add(1)
            .capacity("object generation exhausted");
        if before.zone == Zone::Graveyard {
            self.forget_graveyard_cast_permission(before.reference);
        }
        if (before.zone == Zone::Library
            && self.state.players[before.owner.index()].library.last() == Some(&id))
            || destination == Zone::Library
        {
            self.library_top_changed(before.owner);
        }
        let revealed_top = self.public_library_top(before.owner) == Some(before.reference);
        if before.zone == Zone::Library && revealed_top || destination == Zone::Library {
            self.forget_library_top(before.owner);
        }
        if before.zone == Zone::Hand {
            self.state
                .known_hand
                .retain(|&reference| reference != before.reference);
        }
        if before.zone == Zone::Library {
            let library = &mut self.state.players[before.owner.index()].library;
            let position = library
                .iter()
                .rposition(|&card| card == id)
                .expect("library membership");
            library.remove(position);
        }
        if before.zone == Zone::Exile {
            self.remove_plotted(before.reference);
            self.remove_warped(before.reference);
            self.state
                .exile_permissions
                .retain(|p| p.object != before.reference);
            self.state
                .exile_links
                .retain(|link| link.exiled != before.reference);
        }
        if before.zone == Zone::Stack {
            self.state
                .stack
                .retain(|spell| spell.kind != StackKind::Spell || spell.object != before.reference);
        }
        if before.zone == Zone::Battlefield {
            self.capture_delayed_copy_source_departure(&before);
            self.forget_temporary_control(before.reference);
            self.forget_role(before.reference);
            self.forget_base_stats(before.reference);
            self.forget_animations(before.reference);
            self.forget_creature_types(before.reference);
            self.forget_live_static_source(before.reference);
            self.forget_ability_source(before.reference);
            self.forget_hand_size_source(before.reference);
            // Abilities use the source's actual last-known keywords if it leaves,
            // including gains made after that ability was announced.
            for item in self
                .state
                .stack
                .iter_mut()
                .chain(&mut self.state.pending_triggers)
                .chain(&mut self.state.ordering_triggers)
                .chain(self.state.resolving.iter_mut().map(|r| &mut r.item))
                .chain(self.state.announcing.iter_mut().map(|a| &mut a.item))
                .chain(
                    self.state
                        .mana_announcement_stack
                        .iter_mut()
                        .flat_map(|v| v.iter_mut())
                        .map(|f| &mut f.announcement.item),
                )
            {
                if item.needs_target_controller_lki()
                    && item
                        .targets
                        .iter()
                        .any(|bound| bound.object == Some(before.reference))
                {
                    let history = item
                        .target_controller_lki
                        .get_or_insert_with(|| Box::new(Vec::new()));
                    if !history
                        .iter()
                        .any(|(object, _)| *object == before.reference)
                    {
                        history.push((before.reference, before.controller));
                    }
                }
                if item.object == before.reference {
                    item.source_keywords = before.keywords;
                    item.source_tapped = before.tapped;
                    item.source_attachment = before.attached_to;
                    item.source_facts = DamageFacts::from_view(&before);
                }
            }
            if let Some(data) = &mut self.state.evidence_data {
                data.sources.retain(|&source| source != id);
            }
            if let Some(data) = &mut self.state.explore_data {
                data.sources.retain(|&source| source != id);
            }
            self.state.static_sources.retain(|&source| source != id);
            self.state.cast_sources.retain(|&source| source != id);
            self.state.attack_sources.retain(|&source| source != id);
            self.state.tap_sources.retain(|&source| source != id);
            self.state.damage_sources.retain(|&source| source != id);
            self.state.discard_sources.retain(|&source| source != id);
            self.state.draw_sources.retain(|&source| source != id);
            self.state.step_sources.retain(|&source| source != id);
            self.state.life_sources.retain(|&source| source != id);
            self.state.counter_sources.retain(|&source| source != id);
            self.state.death_sources.retain(|&source| source != id);
            self.forget_ability_target_source(id);
            self.state.attachment_sources.retain(|&source| source != id);
            self.state.attackers.retain(|attacker| attacker.card != id);
            self.state.blocking.retain(|&blocker| blocker != id);
            self.state
                .blocks
                .retain(|&(blocker, attacker)| blocker != id && attacker != id);
            self.state.selected_attackers.retain(|a| a.card != id);
            for attack in self
                .state
                .attackers
                .iter_mut()
                .chain(&mut self.state.selected_attackers)
            {
                if attack.defender == Some(CombatDefender::Planeswalker(before.reference)) {
                    attack.defender = None;
                }
            }
            self.state
                .selected_blocks
                .retain(|&(blocker, attacker)| blocker != id && attacker != id);
            self.state.first_strikers.retain(|&card| card != id);
            // A surviving attacker remains blocked after its last blocker leaves.
        }
        // Every departure observer above used the old complete layer-one facts.
        // The new incarnation carries the underlying physical definition only.
        self.forget_temporary_copy(before.reference);
        self.forget_copy_grant_origin(before.reference);
        let obj = &mut self.state.objects[id as usize];
        // CR608.3f/707.10f: a resolving permanent-spell copy becomes a token.
        // This is an entry, not token creation, so creation replacements do not apply.
        if before.zone == Zone::Stack && destination == Zone::Battlefield && obj.is_spell_copy() {
            obj.copy_kind = CopyKind::Token;
        }
        obj.card = if destination == Zone::Battlefield && before.zone == Zone::Stack {
            before.card
        } else if !obj.is_token() && !obj.is_spell_copy() {
            obj.card.front_face()
        } else {
            obj.card
        };
        obj.zone = destination;
        // Only stack and battlefield objects have independent control.
        obj.controller = if matches!(destination, Zone::Battlefield | Zone::Stack) {
            entry_controller.unwrap_or(before.controller)
        } else {
            obj.owner
        };
        obj.flashback = false;
        obj.generation = generation;
        obj.tapped = tapped;
        obj.summoning_sick = destination == Zone::Battlefield;
        obj.damage = 0;
        obj.damaged_this_turn = false;
        obj.dealt_damage_this_turn = false;
        obj.deathtouch_since_sba = false;
        obj.growth = Stats::default();
        obj.gained_keywords = Keywords::NONE;
        obj.counters = counters.hot();
        obj.used_abilities = 0;
        obj.attached_to = None;
        // This ordered index enumerates library cards, not transient tokens.
        // Tokens still enter the actual zone and cease only at the next SBA;
        // CR111.6/111.8 prevent looking at, drawing, or moving them as cards.
        if destination == Zone::Library && !obj.is_token() {
            self.state.players[obj.owner.index()].library.push(id);
        }
        let entry_identity = (obj.controller, obj.card);
        self.advance_copy_recipe(
            before.reference,
            ObjectRef {
                card: id,
                generation,
            },
        );
        if before.zone == Zone::Stack {
            self.continue_stack_keywords(before.reference, self.object_ref(id), destination);
        }
        let entered_with_counters = counters.has_any();
        let entered_with_plus_one = counters.plus_one > 0;
        self.set_counters(id, counters);
        if destination == Zone::Battlefield {
            if entered_with_counters {
                self.record_counter_placement(
                    self.object_ref(id),
                    self.state.objects[id as usize].controller,
                    entered_with_plus_one,
                );
            }
            let (owner, _) = entry_identity;
            self.record_nonland_entry(owner, self.current_kinds(id));
            self.register_battlefield_sources(id);
        } else if destination == Zone::Graveyard {
            self.register_graveyard_sources(id);
        }
        if revealed_top && before.zone == Zone::Library && destination == Zone::Hand {
            let reference = self.object_ref(id);
            if !self.state.known_hand.contains(&reference) {
                self.state.known_hand.push(reference);
            }
        }
        if plan.shuffle {
            self.reveal_and_shuffle_zone_card(&before);
        }
        self.copy_characteristic_zone_commit(&before, self.object_ref(id), destination);
        ZoneChange {
            cast_event: None,
            entry_counter_player: (destination == Zone::Battlefield).then_some(entry_identity.0),
            graveyard_observers,
            returned_hand_observers,
            was_cast: false,
            gift: None,
            before,
            after: self.object_ref(id),
            destination,
            moved: true,
            sacrificed_by: None,
            offspring_paid: false,
            kicked: false,
            bargained: false,
            x: 0,
            evidence_collected: false,
        }
    }

    /// Complete the turn-based declaration before dispatching attack events.
    /// Keeping this boundary explicit also permits testing live trigger guards.
    fn commit_attackers_with(&mut self, actor: Player, mut event: impl FnMut(&mut Self, CardId)) {
        self.state.attack_declared = !self.state.selected_attackers.is_empty();
        self.state.players[actor.index()].attacked_this_turn |= self.state.attack_declared;
        let declared = std::mem::take(&mut self.state.selected_attackers);
        let alone = (declared.len() == 1).then(|| self.object_ref(declared[0].card));
        // Determine vigilance before any simultaneous tap changes characteristics.
        let to_tap: Vec<_> = declared
            .iter()
            .filter(|a| !self.keywords(a.card).contains(Keywords::VIGILANCE))
            .map(|a| a.card)
            .collect();
        for attack in &declared {
            let reference = self.object_ref(attack.card);
            if !self.state.attacked_this_turn.contains(&reference) {
                self.state.attacked_this_turn.push(reference);
            }
        }
        self.state.attackers.extend(declared.iter().cloned());
        self.tap_objects(&to_tap);
        self.record_attack_turn_facts(actor, &declared);
        for attack in &declared {
            event(self, attack.card);
        }
        if self.state.attack_declared {
            self.declaration_triggers(actor, alone, &declared);
        }
    }

    fn give_priority(&mut self) {
        if self.miracle_choice_pending() {
            return;
        }
        self.state.pending = Pending::Priority;
        self.state.priority = self.state.active;
        self.state.passes = 0;
    }

    // Both normal zone moves and token batches record actual entry once. All
    // entries in a simultaneous batch are recorded before entry observers run.
    fn record_nonland_entry(&mut self, controller: Player, kinds: u16) {
        if kinds & 2 == 0 {
            let count =
                &mut self.state.players[controller.index()].nonland_permanents_entered_this_turn;
            *count = count
                .checked_add(1)
                .capacity("nonland entry history exhausted");
        }
    }

    fn begin_turn(&mut self) {
        self.begin_player_control_turn();
        self.expire_temporary_copies(false);
        self.reset_copy_turn_uses();
        self.prune_modal_history();
        self.state.entered_this_turn = None;
        self.state
            .exile_permissions
            .retain(|p| p.expiry != ExilePlayExpiry::StartOfNextTurn(self.state.active));
        for permission in &mut self.state.exile_permissions {
            if permission.expiry == ExilePlayExpiry::EndOfNextTurn
                && permission.player == self.state.active
            {
                permission.expiry = ExilePlayExpiry::Turn(self.state.turn);
            }
        }
        if let Some(bans) = &mut self.state.hand_play_bans {
            bans.retain(|&owner| owner != self.state.active);
            if bans.is_empty() {
                self.state.hand_play_bans = None;
            }
        }
        self.clear_crew_history();
        self.expire_spell_copies();
        self.state.triggered_this_turn.clear();
        self.state.ability_resolutions = None;
        self.state.used_free_hand_permissions = None;
        self.state.creature_damage_history = None;
        self.state.tap_history = None;
        self.state.counter_placement_history = None;
        self.state.cast_entry_counters = None;
        self.state.combat_rules = None;
        self.state.attacked_this_turn.clear();
        self.state.cards_exiled_this_turn = false;
        self.state.creatures_died_this_turn = 0;
        self.state.graveyard_entries = None;
        for player in &mut self.state.players {
            player.turn_facts = TurnFactsView::default();
            player.nonland_permanents_entered_this_turn = 0;
            player.spells_cast_this_turn = 0;
            player.instant_sorcery_cast_this_turn = 0;
            player.cards_drawn_this_turn = 0;
            player.cards_discarded_this_turn = 0;
            player.card_left_graveyard_this_turn = false;
            player.controlled_creature_died_this_turn = false;
            player.life_gain_events_this_turn = 0;
            player.life_gained_this_turn = 0;
            player.life_lost_this_turn = 0;
            player.attacked_this_turn = false;
        }
        self.state.land_played = false;
        self.state.continuous_land_plays_used = 0;
        self.state.attack_declared = false;
        self.state.attackers.clear();
        self.state.blocking.clear();
        self.state.blocks.clear();
        self.state.first_strikers.clear();
        if let Some(rows) = &mut self.state.borrowed_uses {
            rows.retain(|row| row.exhaust || row.turn == self.state.turn);
            if rows.is_empty() {
                self.state.borrowed_uses = None;
            }
        }
        for obj in &mut self.state.objects {
            if obj.used_abilities != 0 {
                obj.used_abilities &= obj.card.exhaust_mask();
            }
            obj.damaged_this_turn = false;
            obj.dealt_damage_this_turn = false;
            if obj.controller == self.state.active && obj.zone == Zone::Battlefield {
                obj.summoning_sick = false;
            }
        }
        self.untap_active_player();
        self.begin_step(Step::Upkeep);
        self.give_priority();
    }

    fn advance_step(&mut self) {
        // Firebending's red remains through each combat step, including the
        // end-of-combat priority window, and expires when that combat ends.
        let keep_combat = matches!(
            self.state.step,
            Step::BeginCombat
                | Step::Attackers
                | Step::Blockers
                | Step::FirstStrikeDamage
                | Step::CombatDamage
        );
        self.clear_step_mana(keep_combat, self.state.step != Step::Cleanup);
        match self.state.step {
            Step::Upkeep => {
                if self.state.turn == 1 {
                    self.begin_step(Step::Main1);
                } else {
                    self.begin_step(Step::Draw);
                    self.draw(self.state.active);
                }
                self.give_priority();
            }
            Step::Draw => {
                self.begin_step(Step::Main1);
                self.give_priority();
            }
            Step::Main1 => {
                self.begin_step(Step::BeginCombat);
                self.give_priority();
            }
            Step::BeginCombat => {
                self.state.step = Step::Attackers;
                self.state.pending = Pending::Attackers;
            }
            Step::Attackers => {
                if !self.state.attack_declared {
                    self.begin_step(Step::EndCombat);
                    self.give_priority();
                } else {
                    self.state.step = Step::Blockers;
                    self.state.pending = Pending::Blockers;
                }
            }
            Step::Blockers => {
                self.state.first_strikers = self
                    .combatants()
                    .into_iter()
                    .filter(|&id| {
                        let keywords = self.keywords(id);
                        keywords.contains(Keywords::FIRST_STRIKE)
                            || keywords.contains(Keywords::DOUBLE_STRIKE)
                    })
                    .collect();
                self.state.step = if self.state.first_strikers.is_empty() {
                    Step::CombatDamage
                } else {
                    Step::FirstStrikeDamage
                };
                self.prepare_damage();
            }
            Step::FirstStrikeDamage => {
                self.state.step = Step::CombatDamage;
                self.prepare_damage();
            }
            Step::CombatDamage => {
                self.begin_step(Step::EndCombat);
                self.give_priority();
            }
            Step::EndCombat => {
                self.state.attackers.clear();
                self.state.blocking.clear();
                self.state.attack_declared = false;
                self.state.blocks.clear();
                self.state.first_strikers.clear();
                if self.take_additional_combat() {
                    self.begin_step(Step::BeginCombat);
                } else {
                    self.begin_step(Step::Main2);
                }
                self.give_priority();
            }
            Step::Main2 => {
                self.begin_step(Step::End);
                self.give_priority();
            }
            Step::End | Step::Cleanup => {
                self.state.step = Step::Cleanup;
                if self.above_hand_limit(self.state.active, 0) {
                    self.state.pending = Pending::Discard;
                } else {
                    self.finish_cleanup();
                }
            }
            Step::Pregame => unreachable!(),
        }
    }

    fn finish_cleanup(&mut self) {
        if let Some(rules) = &mut self.state.combat_rules {
            rules.must_be_blocked.clear();
            rules.additional_combats = 0;
        }
        self.state.casting_prohibited = false;
        self.state.damage_prevention_prohibited = false;
        for player in &mut self.state.players {
            player.hexproof_until_end_turn = false;
        }
        // Until-end-of-turn abilities expire before any cleanup-induced SBA death.
        self.expire_spell_copies();
        self.expire_temporary_controls();
        self.state.self_granted_abilities = None;
        if let Some(grants) = &mut self.state.temporary_abilities {
            grants.retain(|grant| grant.permanent);
            if grants.is_empty() {
                self.state.temporary_abilities = None;
            }
        }
        self.expire_base_stats();
        let expired_animations = self.expire_animations();
        self.expire_graveyard_cast_permissions();
        self.state.turn_cast_grants = None;
        self.expire_keyword_effects();
        if let Some(roles) = &mut self.state.roles {
            roles.toughness_combat_damage = [false; 2];
        }
        self.state.death_exile_replacements = None;
        for player in &mut self.state.players {
            player.extra_land_plays_remaining = 0;
            player.extra_land_plays_granted = 0;
        }
        let expired_vehicles = self.expire_vehicle_animation();
        self.state.saddled.clear();
        self.state.exile_permissions.retain(
            |p| !matches!(p.expiry, ExilePlayExpiry::Turn(turn) if turn <= self.state.turn),
        );
        // Damage removal and expiration of until-end-of-turn effects are simultaneous.
        for obj in &mut self.state.objects {
            obj.damage = 0;
            obj.deathtouch_since_sba = false;
            obj.growth = Stats::default();
            obj.gained_keywords = Keywords::NONE;
        }
        let expired_copies = self.expire_temporary_copies(true);
        // Losing temporary growth can expose lethal negative counters. Apply
        // SBAs before advancing the turn; CR514.3 gives an extra priority cycle.
        if !self.state.pending_triggers.is_empty()
            || self
                .allowed_game_losses(
                    self.state
                        .players
                        .each_ref()
                        .map(|p| p.life <= 0 || p.poison_counters >= 10 || p.attempted_empty_draw),
                )
                .into_iter()
                .any(|lost| lost)
            || (expired_vehicles || expired_animations || expired_copies) && self.sba_required()
            || self.state.objects.iter().enumerate().any(|(id, obj)| {
                obj.zone == Zone::Battlefield
                    && self.is_creature(id as CardId)
                    && self.stats(id as CardId).toughness <= 0
            })
        {
            self.give_priority();
            self.state_based_actions();
            return;
        }
        self.clear_step_mana(false, false);
        self.advance_to_actual_turn();
        self.state.turn = self
            .state
            .turn
            .checked_add(1)
            .capacity("turn count exhausted");
        // Per-turn permissions persist through repeated cleanup priority, then reset
        // only when the turn actually advances (including an effect ending it).
        self.state.graveyard_type_uses = None;
        self.begin_turn();
    }

    fn damage_source(&self, object: CardId) -> DamageSource {
        let obj = &self.state.objects[object as usize];
        DamageSource {
            facts: self.damage_facts(object),
            object,
            reference: self.object_ref(object),
            controller: obj.controller,
            keywords: self.keywords(object),
        }
    }

    fn queue_hit(&mut self, source: DamageSource, target: Target, amount: i128, combat: bool) {
        if amount > 0 {
            self.state.hits.push(DamageEvent {
                preventable: true,
                source,
                target,
                amount,
                combat,
            });
        }
    }

    fn apply_damage_batch(&mut self, events: &[DamageEvent]) -> Vec<LifeGainEvent> {
        if self.damage_replacement_candidates_exist(events) {
            self.stage_damage_replacements(events);
            Vec::new()
        } else {
            self.commit_damage_batch(events)
        }
    }

    fn commit_damage_batch(&mut self, events: &[DamageEvent]) -> Vec<LifeGainEvent> {
        // Avoid allocating on the ordinary, unprevented damage path.
        let prevented = |event: &DamageEvent| {
            event.preventable
                && !self.state.damage_prevention_prohibited
                && ((event.combat
                    && (event.source.keywords.contains(Keywords::PREVENT_COMBAT_BY)
                        || matches!(event.target, Target::Card(id) if self.keywords(id).contains(Keywords::PREVENT_COMBAT_TO))))
                    || matches!(event.target, Target::Card(id) if self.keywords(id).contains(Keywords::PROTECTION_EVERYTHING) || (self.keywords(id).contains(Keywords::PROTECTION_INSTANT_SORCERY) && event.source.facts.kinds & (4 | 8) != 0)))
        };
        let dealt = events.iter().any(prevented).then(|| {
            events
                .iter()
                .copied()
                .filter(|event| !prevented(event))
                .collect::<Vec<_>>()
        });
        let events = dealt.as_deref().unwrap_or(events);
        let mut losses = [0i128; 2];
        let mut gains: Vec<LifeGainEvent> = Vec::new();
        let mut gain_indices: std::collections::BTreeMap<DamageSource, usize> =
            std::collections::BTreeMap::new();
        for event in events {
            if event.amount <= 0 {
                continue;
            }
            match event.target {
                Target::Player(p) => {
                    self.lose_life_unobserved(p, event.amount);
                    losses[p.index()] = losses[p.index()]
                        .checked_add(event.amount)
                        .capacity("damage life-loss batch exhausted");
                }
                Target::Card(id) => {
                    let creature = self.is_creature(id);
                    let planeswalker = self.current_kinds(id) & 64 != 0;
                    let obj = &mut self.state.objects[id as usize];
                    if planeswalker {
                        let removed = u128::try_from(event.amount)
                            .expect("positive damage")
                            .min(u128::from(obj.counters.loyalty))
                            as u64;
                        obj.counters.loyalty -= removed;
                    }
                    if creature {
                        obj.damage = obj
                            .damage
                            .checked_add(event.amount)
                            .capacity("damage representation exhausted");
                    }
                    obj.damaged_this_turn = true;
                    obj.deathtouch_since_sba |=
                        creature && event.source.keywords.contains(Keywords::DEATHTOUCH);
                }
                Target::Stack(_) | Target::Spell(_) => {
                    unreachable!("damage cannot target stack objects")
                }
            }
            // An ability can retain a departed source's last-known reference;
            // do not transfer this history to its new zone incarnation.
            if let Some(source) = self
                .state
                .objects
                .get_mut(event.source.reference.card as usize)
                && source.generation == event.source.reference.generation
            {
                source.dealt_damage_this_turn = true;
            }
            if event.source.keywords.contains(Keywords::LIFELINK) {
                // One source damaging multiple objects simultaneously causes a
                // single life-gain event; distinct sources remain distinct.
                if let Some(&index) = gain_indices.get(&event.source) {
                    gains[index].amount = gains[index]
                        .amount
                        .checked_add(event.amount)
                        .capacity("life gain representation exhausted");
                } else {
                    gain_indices.insert(event.source, gains.len());
                    gains.push(LifeGainEvent {
                        source: event.source,
                        amount: event.amount,
                    });
                }
            }
        }
        for player in [Player::P0, Player::P1] {
            self.life_lost_event(player, losses[player.index()]);
        }
        for gain in &gains {
            self.gain_life(gain.source.controller, gain.amount);
        }
        self.record_combat_and_excess_damage(events);
        self.creature_damage_events(events);
        self.creature_received_damage_triggers(events);
        self.player_damage_triggers(events);
        // Trigger events are recorded before SBAs, after the whole damage batch.
        gains
    }

    fn combatants(&self) -> Vec<CardId> {
        self.state
            .attackers
            .iter()
            .map(|attacker| attacker.card)
            .chain(self.state.blocking.iter().copied())
            .filter(|&id| self.on_battlefield(id))
            .collect()
    }

    fn assigns_this_step(&self, id: CardId) -> bool {
        if self.state.step == Step::FirstStrikeDamage {
            self.state.first_strikers.contains(&id)
        } else {
            !self.state.first_strikers.contains(&id)
                || self.keywords(id).contains(Keywords::DOUBLE_STRIKE)
        }
    }

    fn lethal_assignment(&self, source: DamageSource, blocker: CardId) -> i128 {
        let obj = &self.state.objects[blocker as usize];
        let remaining = self
            .stats(blocker)
            .toughness
            .checked_sub(obj.damage)
            .capacity("damage assignment representation exhausted")
            .max(0);
        if source.keywords.contains(Keywords::DEATHTOUCH) {
            remaining.min(1)
        } else {
            remaining
        }
    }

    fn combat_defender_target(&self, defender: Option<CombatDefender>) -> Option<Target> {
        match defender? {
            CombatDefender::Player(player) => Some(Target::Player(player)),
            CombatDefender::Planeswalker(reference) => self
                .state
                .objects
                .get(reference.card as usize)
                .filter(|o| {
                    o.zone == Zone::Battlefield
                        && o.generation == reference.generation
                        && self.current_kinds(reference.card) & 64 != 0
                })
                .map(|_| Target::Card(reference.card)),
        }
    }

    /// Assignment rule only. Actual characteristic power and noncombat damage are unchanged.
    pub(super) fn combat_assignment_amount(&self, id: CardId) -> i128 {
        let stats = self.stats(id);
        if self.state.roles.as_ref().is_some_and(|r| {
            r.toughness_combat_damage[self.state.objects[id as usize].controller.index()]
        }) && stats.toughness > stats.power
        {
            stats.toughness.max(0)
        } else {
            stats.power.max(0)
        }
    }
    fn prepare_damage(&mut self) {
        self.state.hits.clear();
        self.state.damage_choices.clear();
        // Clone only the declaration list so construction may append events.
        for attacker in self.state.attackers.clone() {
            if !self.on_battlefield(attacker.card) {
                continue;
            }
            let blockers: Vec<_> = self
                .state
                .blocks
                .iter()
                .filter(|&&(b, a)| a == attacker.card && self.on_battlefield(b))
                .map(|&(b, _)| b)
                .collect();
            for &blocker in &blockers {
                if self.assigns_this_step(blocker) {
                    let source = self.damage_source(blocker);
                    let amount = self.combat_assignment_amount(blocker);
                    self.queue_hit(source, Target::Card(attacker.card), amount, true);
                }
            }
            if !self.assigns_this_step(attacker.card) {
                continue;
            }
            let source = self.damage_source(attacker.card);
            let power = self.combat_assignment_amount(attacker.card);
            let defender = self.combat_defender_target(attacker.defender);
            let trample = source.keywords.contains(Keywords::TRAMPLE) && defender.is_some();
            if !attacker.blocked || (trample && blockers.is_empty()) {
                if let Some(target) = defender {
                    self.queue_hit(source, target, power, true);
                }
            } else if !blockers.is_empty() && power > 0 {
                self.state.damage_choices.push(DamageChoice {
                    source,
                    defender: attacker.defender,
                    blockers,
                    remaining: power,
                    trample,
                    prior_blockers_lethal: true,
                });
            }
        }
        self.advance_damage_choices();
    }

    fn advance_damage_choices(&mut self) {
        while let Some(choice) = self.state.damage_choices.last() {
            let forced = choice.remaining == 0
                || (choice.blockers.len() == 1
                    && (!choice.trample
                        || !choice.prior_blockers_lethal
                        || choice.remaining
                            <= self.lethal_assignment(choice.source, choice.blockers[0])));
            if !forced {
                self.state.pending = Pending::Damage;
                return;
            }
            let choice = self.state.damage_choices.pop().unwrap();
            self.queue_hit(
                choice.source,
                Target::Card(choice.blockers[0]),
                choice.remaining,
                true,
            );
        }
        self.deal_combat_damage();
    }

    fn deal_combat_damage(&mut self) {
        let hits = std::mem::take(&mut self.state.hits);
        self.give_priority();
        self.apply_damage_batch(&hits);
        self.advance_replacements(false);
    }

    /// Player-facing information only. Unrevealed opponent hands and RNG state
    /// are absent. Explicit library looks/searches and partial declarations are
    /// projected only to their actor; known revealed hand incarnations are public.
    pub fn observe(&self, viewer: Player) -> Observation {
        let s = &self.state;
        let owns_choice = self
            .semantic_actor()
            .is_some_and(|p| self.shares_information(viewer, p));
        let makes_decision = self.actor() == Some(viewer);
        let choice_player = self.choice_information_player(viewer);
        let query = self.characteristic_query();
        let objects = s
            .objects
            .iter()
            .enumerate()
            .filter(|(i, o)| {
                self.can_view_hideaway(viewer, self.object_ref(*i as CardId))
                    && o.zone != Zone::Outside
                    && o.zone != Zone::Library
                    && (o.zone != Zone::Hand
                        || self.shares_information(viewer, o.owner)
                        || s.known_hand.contains(&self.object_ref(*i as CardId)))
            })
            .map(|(i, _)| query.object_view(i as CardId))
            .collect();
        Observation {
            player_control: self.player_control_view(),
            library_cast_look: self.library_cast_look_view(viewer),
            zone_search: self.zone_search_view(viewer),
            turn_facts: self.turn_facts_view(),
            paradigms: self.paradigm_views(),
            resolved_spell_names: self.resolved_spell_name_views(),
            preparations: s
                .preparations
                .as_ref()
                .map_or_else(Vec::new, |rows| rows.to_vec()),
            library_piles: self.library_piles_view(viewer),
            mana_rollback: self.mana_rollback_view(viewer),
            cast_costs: self
                .state
                .announcing
                .as_ref()
                .filter(|a| self.shares_information(viewer, a.item.controller))
                .and_then(|a| a.cast_costs.as_deref())
                .cloned(),
            casualty_declarations: self
                .state
                .announcing
                .as_ref()
                .filter(|a| self.shares_information(viewer, a.item.controller))
                .and_then(|a| a.casualty.as_deref())
                .map_or_else(Vec::new, |v| v.clone()),
            keep_types: self.keep_types_view(),
            hand_choice: self.hand_choice_view(viewer),
            hand_deployment: self.public_hand_deployment_view(),
            linked_exile: s.linked_choices.as_ref().map_or_else(Vec::new, |c| {
                c.links
                    .iter()
                    .copied()
                    .filter(|link| self.can_view_hideaway(viewer, link.exiled))
                    .collect()
            }),
            casting_prohibited: s.casting_prohibited || self.static_casting_prohibited(viewer),
            damage_prevention_prohibited: s.damage_prevention_prohibited,
            replacement_choice: self.replacement_choice(viewer),
            zone_replacement_choice: self.zone_choice_view(viewer),
            zone_reveals: self.zone_reveals(),
            created_token_choice: self.created_token_choice(),
            mana_lots: self.mana_lot_views(),
            mana_lot_spend: owns_choice.then(|| self.mana_lot_spend_range()).flatten(),
            mana_payment: owns_choice.then(|| self.mana_payment_domain()).flatten(),
            endure_choice: if owns_choice {
                match s.pending {
                    Pending::Endure {
                        source,
                        amount,
                        token,
                        ..
                    } => Some(EndureChoiceView {
                        source,
                        amount,
                        token,
                    }),
                    _ => None,
                }
            } else {
                None
            },
            improvise_choice: if owns_choice {
                self.improvise_choice()
            } else {
                None
            },
            convoke_choice: if owns_choice {
                self.convoke_choice()
            } else {
                None
            },
            immediate_blink_return: self
                .state
                .resolving
                .as_ref()
                .and_then(|r| r.immediate_blink_return.as_deref().cloned()),
            delayed_returns: self
                .state
                .delayed_returns
                .as_ref()
                .map_or_else(Vec::new, |rows| {
                    rows.iter().map(|r| r.view.clone()).collect()
                }),
            delayed_main_mana: self
                .state
                .delayed_main_mana
                .as_deref()
                .cloned()
                .unwrap_or_default(),
            delayed_artifact_copies: self
                .state
                .delayed_artifact_copies
                .as_deref()
                .cloned()
                .unwrap_or_default(),
            delayed_sacrifices: self
                .state
                .delayed_sacrifices
                .as_ref()
                .map_or_else(Vec::new, |b| b.iter().map(|d| d.view.clone()).collect()),
            mobilize_choice: if owns_choice {
                self.mobilize_choice()
            } else {
                None
            },
            saddle_choice: if owns_choice {
                self.saddle_choice()
            } else {
                None
            },
            teamwork_choice: if owns_choice {
                self.teamwork_choice()
            } else {
                None
            },
            source_counter_choice: if owns_choice {
                self.source_counter_choice()
            } else {
                None
            },
            x_choice: if owns_choice { self.x_choice() } else { None },
            suspected: s
                .suspected
                .as_ref()
                .map_or_else(Vec::new, |v| v.as_ref().clone()),
            temporary_controls: s
                .temporary_controls
                .as_ref()
                .map_or_else(Vec::new, |v| v.as_ref().clone()),
            self_granted_abilities: self.self_granted_ability_views(),
            temporary_abilities: s
                .temporary_abilities
                .as_ref()
                .map_or_else(Vec::new, |grants| grants.as_ref().clone()),
            death_exile_replacements: s
                .death_exile_replacements
                .as_ref()
                .map_or_else(Vec::new, |rules| rules.as_ref().clone()),
            animations: s
                .roles
                .as_ref()
                .map_or_else(Vec::new, |r| r.animations.clone()),
            cast_entry_counters: s
                .cast_entry_counters
                .as_ref()
                .map_or_else(Vec::new, |rows| rows.iter().copied().collect()),
            counter_placement_history: s
                .counter_placement_history
                .as_ref()
                .map_or_else(Vec::new, |rows| rows.iter().copied().collect()),
            creature_damage_history: s
                .creature_damage_history
                .as_ref()
                .map_or_else(Vec::new, |rows| rows.iter().copied().collect()),
            schema: OBSERVATION_SCHEMA,
            toughness_combat_damage: s.roles.as_ref().and_then(|r| {
                r.toughness_combat_damage
                    .iter()
                    .any(|x| *x)
                    .then_some(r.toughness_combat_damage)
            }),
            additional_combats: s.combat_rules.as_ref().map_or(0, |r| r.additional_combats),
            turn_schedule: s.turn_schedule.as_deref().cloned(),
            must_be_blocked: s
                .combat_rules
                .as_ref()
                .map_or_else(Vec::new, |r| r.must_be_blocked.clone()),
            combat_damaged_players: s
                .combat_rules
                .as_ref()
                .map_or_else(Vec::new, |r| r.damaged_players.clone()),
            creatures_attacked_this_turn: s.attacked_this_turn.len(),
            cards_exiled_this_turn: s.cards_exiled_this_turn,
            creatures_died_this_turn: s.creatures_died_this_turn,
            graveyard_entries_this_turn: s
                .graveyard_entries
                .as_ref()
                .map_or_else(Vec::new, |r| r.to_vec()),
            catalogue: crate::CATALOGUE_VERSION,
            viewer,
            revision: self.revision,
            actor: self.actor(),
            active: s.active,
            turn: s.turn,
            step: s.step,
            decision: match s.pending {
                Pending::MiracleReveal { .. } => "miracle_reveal",
                Pending::ResolutionCasts { .. } => "resolution_casts",
                Pending::ManaLotSpend { .. } => "mana_lot_spend",
                Pending::ZoneReplacement { .. } => "zone_replacement",
                Pending::Replacement { .. } => "replacement",
                Pending::CreatedToken { .. } => "created_token",
                Pending::ResolutionMana { .. } => "resolution_mana",
                Pending::ResolutionLife { .. } => "resolution_life",
                Pending::ConniveOrder { .. } => "connive_order",
                Pending::OpeningHand { .. } => "opening_hand",
                Pending::OptionalSearch(_) => "optional_search",
                Pending::Mulligan(_) => "mulligan",
                Pending::Bottom { .. } => "bottom",
                Pending::Priority => "priority",
                Pending::PlotPayment { .. } => "plot_payment",
                Pending::Role { .. } => "role_rule",
                Pending::Legend { .. } => "legend_rule",
                Pending::Retarget { .. } => "retarget",
                Pending::TargetAmount { .. } => "target_amount",
                Pending::Targets { .. } => "targets",
                Pending::Offspring { .. } => "offspring",
                Pending::Casualty { .. } => "casualty",
                Pending::ManaRollback { .. } => "mana_rollback",
                Pending::CastCosts { .. } => "cast_costs",
                Pending::CasualtyPayment { .. } => "casualty_payment",
                Pending::Bargain { .. } => "bargain",
                Pending::Kicker { .. } => "kicker",
                Pending::Gift { .. } => "gift",
                Pending::Endure { .. } => "endure",
                Pending::Convoke { .. } => "convoke",
                Pending::Improvise { .. } => "improvise",
                Pending::Mobilize { .. } => "mobilize",
                Pending::TeamworkIntent { .. } => "teamwork_intent",
                Pending::TeamworkPayment { .. } => "teamwork_payment",
                Pending::Saddle { .. } => "saddle",
                Pending::Crew { .. } => "crew",
                Pending::Explore { .. } => "explore",
                Pending::ResolutionX { .. } => "resolution_x",
                Pending::SourceCounterAmount { .. } => "source_counter_amount",
                Pending::X { .. } => "x",
                Pending::EvidenceIntent { .. } => "evidence_intent",
                Pending::Evidence { .. } => "evidence",
                Pending::SpellCost { .. } => "spell_cost",
                Pending::ActivationCost { .. } => "activation_cost",
                Pending::Modes { .. } => "spell_mode",
                Pending::AdditionalCost { .. } => "additional_cost",
                Pending::PayCosts { .. } => "pay_costs",
                Pending::OrderTriggers { .. } => "order_triggers",
                Pending::EffectOptional { .. } => "optional_effect",
                Pending::TargetLibrary { .. } => "target_library",
                Pending::ManaValueParity { .. } => "mana_value_parity",
                Pending::ForageMethod { .. } => "forage_method",
                Pending::Forage { .. } => "forage",
                Pending::EffectMana { .. } => "mana_choice",
                Pending::EffectLibrary { .. } => "library_choice",
                Pending::EquipmentAttachment { .. } => "equipment_attachment",
                Pending::EffectLibrarySelection { .. } => "library_selection",
                Pending::HandCardChoice { .. } => "hand_card_choice",
                Pending::RevealedDiscard { .. } => "revealed_discard",
                Pending::RecoverMilled { .. } => "recover_milled",
                Pending::MilledSelection { .. } => "milled_selection",
                Pending::LibraryGraveyardSearch { .. } => "library_graveyard_search",
                Pending::EffectSearch { .. } => {
                    if s.resolving
                        .as_ref()
                        .and_then(|r| r.search.as_ref())
                        .is_some_and(|choice| choice.battlefield.is_some())
                    {
                        "library_search_destination"
                    } else {
                        "library_search"
                    }
                }
                Pending::EffectDiscard { .. } => "effect_discard",
                Pending::HandDefender { .. } => "hand_defender",
                Pending::TokenChoice { .. } => "token_choice",
                Pending::EffectChoice { .. } => "effect_choice",
                Pending::EntryTimestamp { .. } => "entry_timestamps",
                Pending::LinkedChoice { .. } => "linked_choice",
                Pending::EntryValue { .. } => "entry_value",
                Pending::LandCardName { .. } => "land_card_name",
                Pending::EntryLife { .. } => "entry_life",
                Pending::EntryReplacement { .. } => "entry_replacement",
                Pending::AuraEntry { .. } => "aura_entry",
                Pending::Attackers => "attackers",
                Pending::Blockers => "blockers",
                Pending::Damage => "damage",
                Pending::Discard => "discard",
                Pending::Terminal => "terminal",
            },
            players: std::array::from_fn(|i| PlayerView {
                turn_cast_grants: self.player_cast_grants_view(i),
                turns_to_skip: s.turn_skips.as_ref().map_or(0, |v| v[i]),
                hexproof_until_end_turn: s.players[i].hexproof_until_end_turn,
                extra_land_plays_remaining: s.players[i].extra_land_plays_remaining,
                no_maximum_hand_size: self.no_maximum_hand_size(if i == 0 {
                    Player::P0
                } else {
                    Player::P1
                }),
                nonland_permanents_entered_this_turn: s.players[i]
                    .nonland_permanents_entered_this_turn,
                spells_cast_this_turn: s.players[i].spells_cast_this_turn,
                cards_drawn_this_turn: s.players[i].cards_drawn_this_turn,
                cards_discarded_this_turn: s.players[i].cards_discarded_this_turn,
                card_left_graveyard_this_turn: s.players[i].card_left_graveyard_this_turn,
                controlled_creature_died_this_turn: s.players[i].controlled_creature_died_this_turn,
                life_gain_events_this_turn: s.players[i].life_gain_events_this_turn,
                life_gained_this_turn: s.players[i].life_gained_this_turn,
                life_lost_this_turn: s.players[i].life_lost_this_turn,
                attacked_this_turn: s.players[i].attacked_this_turn,
                life: s.players[i].life,
                poison_counters: s.players[i].poison_counters,
                mana: s.players[i].mana,
                combat_red_mana: s.players[i].combat_red_mana,
                library_count: s.players[i].library.len(),
                hand_count: self.hand_count(if i == 0 { Player::P0 } else { Player::P1 }),
            }),
            milled_choice: self.milled_selection_view(viewer),
            recovery_choice: match s.pending {
                Pending::RecoverMilled { player, filter } => Some(RecoveryChoiceView {
                    player,
                    filter,
                    milled: s
                        .resolving
                        .as_ref()
                        .unwrap()
                        .milled
                        .iter()
                        .copied()
                        .filter(|&object| self.public_milled_object(object))
                        .collect(),
                    candidates: s
                        .resolving
                        .as_ref()
                        .unwrap()
                        .milled
                        .iter()
                        .copied()
                        .filter(|&object| self.can_recover_milled(player, filter, object))
                        .collect(),
                }),
                _ => None,
            },
            effect_choice: self.effect_choice_view(viewer),
            entry_value: self.entry_choice_view(viewer),
            aura_entry: self.aura_entry_view(viewer),
            entry_life: self.entry_life_view(viewer),
            entry_replacement: self.entry_tap_view(viewer),
            entry_timestamps: self.entry_timestamp_view(viewer),
            additional_cost: self.additional_cost_view(viewer),
            forage: self.forage_view(viewer),
            evidence: self.evidence_view(viewer),
            last_evidence: s.evidence_data.as_ref().and_then(|d| d.last.clone()),
            objects,
            modal_history: s
                .modal_history
                .as_ref()
                .map_or_else(Vec::new, |rows| rows.to_vec()),
            exile_links: s.exile_links.clone(),
            spell_characteristics: self.spell_characteristics(),
            exile_permissions: s.exile_permissions.clone(),
            borrowed_abilities: self.borrowed_ability_views(),
            cascade: s
                .resolving
                .as_ref()
                .and_then(|r| r.cascade.as_deref().cloned()),
            miracle: self.miracle_choice_view(viewer),
            resolution_free_casts: s
                .resolution_free_casts
                .as_deref()
                .filter(|f| f.origin != Zone::Library || self.shares_information(viewer, f.player))
                .cloned(),
            graveyard_type_uses: s
                .graveyard_type_uses
                .as_ref()
                .map_or_else(Vec::new, |rows| rows.iter().copied().collect()),
            graveyard_cast_permissions: s
                .graveyard_cast_permissions
                .as_ref()
                .map_or_else(Vec::new, |rows| rows.to_vec()),
            adventures: s
                .adventures
                .as_ref()
                .map(|a| a.permissions.clone())
                .unwrap_or_default(),
            plotted_cards: s.plot.as_ref().map_or_else(Vec::new, |p| p.cards.clone()),
            crew_choice: self.crew_choice(viewer),
            crewed_vehicles: s.crew.as_ref().map_or_else(Vec::new, |c| c.history.clone()),
            warped_cards: s.warp.as_ref().map_or_else(Vec::new, |w| w.cards.clone()),
            delayed_warp_exiles: s
                .warp
                .as_ref()
                .map_or_else(Vec::new, |w| w.delayed.iter().map(|d| d.view).collect()),
            plot_payment: self
                .plot_payment()
                .filter(|p| self.shares_information(viewer, p.controller))
                .copied(),
            triggered_this_turn: s.triggered_this_turn.clone(),
            ability_resolutions: s
                .ability_resolutions
                .as_ref()
                .map_or_else(Vec::new, |rows| rows.to_vec()),
            role_choices: s
                .sba_batch
                .as_ref()
                .map_or_else(Vec::new, |b| b.roles.clone()),
            base_stat_effects: s
                .roles
                .as_ref()
                .map_or_else(Vec::new, |r| r.base_stats.clone()),
            live_static_sources: self
                .state
                .roles
                .as_ref()
                .map_or_else(Vec::new, |r| r.live_static.clone()),
            resolved_stat_subjects: s
                .roles
                .as_ref()
                .map_or_else(Vec::new, |r| r.stat_subjects.clone()),
            hand_size_timeline: self.hand_size_timeline(),
            ability_source_timestamps: s
                .roles
                .as_ref()
                .map_or_else(Vec::new, |r| r.ability_timestamps.clone()),
            keyword_effects: self.keyword_effect_views(viewer),
            source_linked_effects: self.source_linked_effect_views(),
            resolved_ability_modifications: self.resolved_ability_modification_views(),
            role_timestamps: s
                .roles
                .as_ref()
                .map_or_else(Vec::new, |r| r.attached.clone()),
            legend_choices: s
                .sba_batch
                .as_ref()
                .map(|batch| {
                    batch
                        .groups
                        .iter()
                        .map(|group| LegendGroupView {
                            controller: group.controller,
                            name: group.name,
                            objects: group.objects.clone(),
                            keep: group.keep,
                        })
                        .collect()
                })
                .unwrap_or_default(),
            stack: s.stack.clone(),
            retarget: self.retarget_view(),
            target_amount: self.target_amount_choice(),
            delayed_spell_copies: self.delayed_spell_copy_view(),
            resolving: s
                .resolving
                .as_ref()
                .map(|resolution| resolution.item.clone()),
            group_move_count: s
                .resolving
                .as_ref()
                .and_then(|resolution| resolution.moved_count),
            suspended_mana_announcements: self.mana_announcement_views(viewer),
            announcing_free_hand_permission: s
                .announcing
                .as_ref()
                .and_then(|a| a.free_hand_permission.as_deref().copied()),
            used_free_hand_permissions: s
                .used_free_hand_permissions
                .as_deref()
                .map_or_else(Vec::new, |rows| rows.clone()),
            announcing: s.announcing.as_ref().map(|announcement| {
                let mut item = announcement.item.clone();
                if !owns_choice {
                    item.targets.clear();
                }
                item
            }),
            announcing_payment: if owns_choice {
                s.announcing.as_ref().and_then(|announcement| {
                    announcement.payment.map(|mana| PendingPaymentView {
                        mana,
                        counters: announcement.counter_payment.clone(),
                    })
                })
            } else {
                None
            },
            resolution_payment: s
                .resolving
                .as_ref()
                .and_then(|r| r.mana_payment.as_deref())
                .map(|p| resolution_mana::ResolutionPaymentView {
                    payer: p.payer,
                    cost: p.cost,
                    alternatives: p.alternatives.to_vec(),
                    life: p.life,
                    victim: p.victim,
                    exile: p.exile,
                }),
            payment_request: self.announcement_payment_request(viewer),
            targeting_group: if owns_choice && matches!(s.pending, Pending::Targets { .. }) {
                s.announcing.as_ref().map(|announcement| announcement.group)
            } else {
                None
            },
            announcing_cost: s.announcing.as_ref().and_then(|a| a.cost),
            cost_choices: if owns_choice
                && matches!(
                    s.pending,
                    Pending::SpellCost { .. } | Pending::ActivationCost { .. }
                ) {
                let a = s.announcing.as_ref().unwrap();
                a.cost_options()
                    .enumerate()
                    .filter(|(_, cost)| self.announcement_cost_fundable(a, *cost))
                    .map(|(index, cost)| SpellCostChoiceView {
                        index: index as u16,
                        cost,
                    })
                    .collect()
            } else {
                Vec::new()
            },
            mode_choices: if owns_choice && matches!(s.pending, Pending::Modes { .. }) {
                let item = &s.announcing.as_ref().unwrap().item;
                let modes = item.modes().expect("modal announcement");
                modes
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| {
                        self.mode_choice_fundable(s.announcing.as_ref().unwrap(), *index)
                    })
                    .map(|(index, mode)| ModeChoiceView {
                        mode: index as u16,
                        text: mode.text,
                        additional_mana: item.source_card.tiered_costs().map(|costs| costs[index]),
                    })
                    .collect()
            } else {
                Vec::new()
            },
            mana_choices: {
                let mut options = Vec::new();
                if owns_choice && let Pending::EffectMana { production, .. } = s.pending {
                    production.visit(|mana| {
                        options.push(mana);
                        false
                    });
                }
                options
            },
            library_selection: self.library_selection_view(viewer),
            entry_transform: self.entry_transform_choice_view(),
            equipment_attachment: self.equipment_attachment_view(),
            target_library: self.target_library_view(viewer),
            last_target_library_move: s.last_target_library_move.as_deref().copied(),
            library_choice: if owns_choice {
                s.resolving
                    .as_ref()
                    .and_then(|r| r.library.as_ref())
                    .map(|choice| LibraryChoiceView {
                        surveil: choice.surveil,
                        remaining: choice
                            .remaining
                            .iter()
                            .map(|&id| query.object_view(id))
                            .collect(),
                        top: choice.top.iter().map(|&id| query.object_view(id)).collect(),
                        bottom: choice
                            .bottom
                            .iter()
                            .map(|&id| query.object_view(id))
                            .collect(),
                        graveyard: choice
                            .graveyard
                            .iter()
                            .map(|&id| query.object_view(id))
                            .collect(),
                    })
            } else {
                None
            },
            optional_search_sequence: s
                .resolving
                .as_ref()
                .and_then(|r| r.optional_searches.as_deref())
                .cloned(),
            search_choice: if owns_choice && matches!(s.pending, Pending::EffectSearch { .. }) {
                s.resolving
                    .as_ref()
                    .and_then(|r| r.search.as_ref())
                    .filter(|choice| choice.battlefield.is_none() && !choice.destination_pending)
                    .map(|choice| SearchChoiceView {
                        filter: choice.spec.filter,
                        minimum: choice.minimum,
                        maximum: choice.spec.maximum,
                        destination: choice.spec.destination,
                        tapped: choice.spec.tapped,
                        reveal: choice.spec.reveal,
                        cards: s.players[choice_player.index()]
                            .library
                            .iter()
                            .map(|&id| query.object_view(id))
                            .collect(),
                        selected: choice.selected.clone(),
                    })
            } else {
                None
            },
            search_destination_choice: s
                .resolving
                .as_ref()
                .and_then(|r| r.search.as_ref())
                .filter(|choice| choice.destination_pending && (choice.spec.reveal || owns_choice))
                .map(|choice| SearchDestinationChoiceView {
                    player: self.semantic_actor().unwrap(),
                    cards: choice
                        .selected
                        .iter()
                        .map(|&id| query.object_view(id))
                        .collect(),
                    destinations: [
                        crate::SearchDestination::Hand,
                        crate::SearchDestination::Graveyard,
                    ],
                }),
            search_distribution: s
                .resolving
                .as_ref()
                .and_then(|r| r.search.as_ref())
                .and_then(|choice| {
                    let battlefield = choice.battlefield.as_ref()?;
                    let crate::SearchDestination::BattlefieldThenHand {
                        battlefield: battlefield_count,
                    } = choice.spec.destination
                    else {
                        return None;
                    };
                    Some(SearchDistributionView {
                        player: s.resolving.as_ref().unwrap().item.controller,
                        battlefield_count,
                        candidates: choice
                            .selected
                            .iter()
                            .map(|&id| query.object_view(id))
                            .collect(),
                        battlefield: battlefield.clone(),
                    })
                }),
            connive_choice: s
                .resolving
                .as_ref()
                .and_then(|r| r.connive.as_ref())
                .map(|c| connive::ConniveChoiceView {
                    amount: c.amount,
                    remaining: c.remaining.clone(),
                    current: c.current,
                }),
            known_hand: s.known_hand.clone(),
            explore_choice: s
                .resolving
                .as_ref()
                .and_then(|r| r.explore.as_ref())
                .map(|c| (**c).clone()),
            last_library_reveal: s.library_top.as_ref().and_then(|s| s.reveal.clone()),
            last_explore: s.explore_data.as_ref().and_then(|d| d.last.clone()),
            known_library_top: std::array::from_fn(|i| {
                self.visible_library_top(viewer, if i == 0 { Player::P0 } else { Player::P1 })
                    .map(|object| RevealedLibraryCard {
                        object,
                        card: s.objects[object.card as usize].card,
                    })
            }),
            waiting_triggers: s.ordering_triggers.clone(),
            effect_discards: s
                .resolving
                .as_ref()
                .map(|resolution| {
                    [Player::P0, Player::P1]
                        .into_iter()
                        .filter(|&p| self.shares_information(viewer, p))
                        .flat_map(|p| resolution.selected_discards[p.index()].iter().copied())
                        .collect()
                })
                .unwrap_or_default(),
            attackers: if owns_choice && s.pending == Pending::Attackers {
                s.selected_attackers.iter().map(|a| a.card).collect()
            } else {
                s.attackers.iter().map(|a| a.card).collect()
            },
            attack_declarations: if owns_choice && s.pending == Pending::Attackers {
                s.selected_attackers.clone()
            } else {
                s.attackers.clone()
            },
            blocking: if owns_choice && s.pending == Pending::Blockers {
                s.selected_blocks.iter().map(|&(b, _)| b).collect()
            } else {
                s.blocking.clone()
            },
            blocks: if owns_choice && s.pending == Pending::Blockers {
                s.selected_blocks.clone()
            } else {
                s.blocks.clone()
            },
            damage_assigned: if owns_choice {
                s.hits.clone()
            } else {
                Vec::new()
            },
            damage_source: if owns_choice {
                s.damage_choices.last().map(|choice| choice.source.object)
            } else {
                None
            },
            damage_assignment: if owns_choice {
                self.damage_assignment_range()
            } else {
                None
            },
            outcome: s.outcome,
            actions: if makes_decision {
                self.legal_actions()
            } else {
                Vec::new()
            },
        }
    }

    /// Bytes occupied by state structs plus initialized vector elements, excluding
    /// allocator overhead, spare capacity, snapshots, and static card definitions.
    pub fn state_bytes(&self) -> usize {
        let s = &self.state;
        std::mem::size_of::<State>()
            + s.player_control.as_ref().map_or(0, |_| {
                std::mem::size_of::<player_control::PlayerControlState>()
            })
            + s.library_top
                .as_ref()
                .map_or(0, |_| std::mem::size_of::<library_top::LibraryTopState>())
            + s.cast_rollback.as_ref().map_or(0, |r| r.bytes())
            + self.mana_lot_bytes()
            + self.mana_announcement_bytes()
            + s.used_free_hand_permissions.as_ref().map_or(0, |rows| {
                std::mem::size_of::<Vec<ObjectRef>>()
                    + rows.len() * std::mem::size_of::<ObjectRef>()
            })
            + turn_schedule::owned_bytes(&s.turn_schedule)
            + s.turn_skips
                .as_ref()
                .map_or(0, |_| std::mem::size_of::<[u64; 2]>())
            + s.zone_rules.as_ref().map_or(0, |r| r.bytes())
            + self.temporary_control_bytes()
            + self.control_bytes()
            + self_granted::owned_bytes(&s.self_granted_abilities)
            + self.temporary_ability_bytes()
            + self.suspected_bytes()
            + self.death_exile_bytes()
            + self.mobilize_bytes()
            + self.return_links_bytes()
            + std::mem::size_of::<Vec<Object>>()
            + std::mem::size_of::<Vec<ObjectRef>>()
            + s.combat_rules.as_ref().map_or(0, |r| r.owned_bytes())
            + (s.attacked_this_turn.len() + s.saddled.len()) * std::mem::size_of::<ObjectRef>()
            + s.saddle_payment.as_ref().map_or(0, |f| {
                std::mem::size_of::<saddle::SaddlePayment>()
                    + f.selected.len() * std::mem::size_of::<ObjectRef>()
            })
            + 2 * std::mem::size_of::<Vec<CardId>>()
            + s.objects.len() * std::mem::size_of::<Object>()
            + s.linked_choices.as_ref().map_or(0, |c| c.bytes())
            + cast_entry_counters::owned_bytes(&s.cast_entry_counters)
            + counter_placement_history::owned_bytes(&s.counter_placement_history)
            + live_counters::owned_bytes(&s.named_counters)
            + entry_choices::owned_bytes(&s.entry_values)
            + power_up::owned_bytes(&s.power_up_uses)
            + tap_history::owned_bytes(&s.tap_history)
            + s.entered_this_turn.as_ref().map_or(0, |r| {
                std::mem::size_of::<Vec<ObjectRef>>() + r.len() * std::mem::size_of::<ObjectRef>()
            })
            + s.pending.owned_bytes()
            + ability_resolutions::owned_bytes(&s.ability_resolutions)
            + copies::owned_bytes(&s.copy_recipes)
            + temporary_copy::owned_bytes(&s.temporary_copies)
            + copy_grant_origins::owned_bytes(&s.copy_grant_origins)
            + s.graveyard_entries.as_ref().map_or(0, |r| {
                std::mem::size_of::<Vec<ObjectRef>>() + r.len() * std::mem::size_of::<ObjectRef>()
            })
            + copy_characteristics::cull_bytes(&s.nightmare_cull)
            + stack_copies::owned_bytes(&s.stack_copy_data)
            + s.delayed_artifact_copies.as_ref().map_or(0, |rows| {
                std::mem::size_of::<Vec<SpellView>>()
                    + rows
                        .iter()
                        .map(|row| std::mem::size_of::<SpellView>() + row.owned_bytes())
                        .sum::<usize>()
            })
            + s.delayed_main_mana.as_ref().map_or(0, |rows| {
                std::mem::size_of::<Vec<SpellView>>()
                    + rows
                        .iter()
                        .map(|row| std::mem::size_of::<SpellView>() + row.owned_bytes())
                        .sum::<usize>()
            })
            + paradigm::owned_bytes(&s.paradigm)
            + faces::owned_bytes(&s.face_changes)
            + modal_history::owned_bytes(&s.modal_history)
            + s.turn_cast_grants
                .as_ref()
                .map_or(0, |_| std::mem::size_of::<[PlayerCastGrantsView; 2]>())
            + cast_permissions::owned_bytes(&s.graveyard_cast_permissions)
            + s.resolution_free_casts.as_ref().map_or(0, |f| {
                std::mem::size_of::<ResolutionFreeCasts>()
                    + f.remaining.len() * std::mem::size_of::<ObjectRef>()
            })
            + s.graveyard_type_uses.as_ref().map_or(0, |rows| {
                std::mem::size_of::<Vec<GraveyardTypeUse>>()
                    + rows.len() * std::mem::size_of::<GraveyardTypeUse>()
            })
            + s.announcing
                .as_ref()
                .and_then(|a| a.power_up_costs.as_ref())
                .map_or(0, |r| r.bytes())
            + s.announcing.as_ref().map_or(0, |a| {
                a.counter_payment.owned_bytes()
                    + a.free_hand_permission
                        .as_ref()
                        .map_or(0, |_| std::mem::size_of::<ObjectRef>())
            })
            + s.exile_links.len() * std::mem::size_of::<ExileLink>()
            + s.exile_permissions.len() * std::mem::size_of::<ExilePlayPermission>()
            + s.borrowed_uses.as_ref().map_or(0, |rows| {
                std::mem::size_of::<Vec<borrowed_activations::Use>>()
                    + rows.len() * std::mem::size_of::<borrowed_activations::Use>()
            })
            + s.borrowed_links.as_ref().map_or(0, |links| {
                std::mem::size_of::<Vec<borrowed_activations::Link>>()
                    + links.len() * std::mem::size_of::<borrowed_activations::Link>()
            })
            + s.hand_play_bans.as_ref().map_or(0, |bans| {
                std::mem::size_of::<Vec<Player>>() + bans.len() * std::mem::size_of::<Player>()
            })
            + ability_target_events::owned_bytes(&s.ability_target_sources)
            + miracle::owned_bytes(&s.miracle)
            + prepared::owned_bytes(&s.preparations)
            + s.adventures.as_ref().map_or(0, |p| p.bytes())
            + s.plot.as_ref().map_or(0, |p| p.bytes())
            + s.crew.as_ref().map_or(0, |c| c.bytes())
            + s.warp.as_ref().map_or(0, |w| w.bytes())
            + s.triggered_this_turn.len() * std::mem::size_of::<TriggerUse>()
            + s.replacements.as_ref().map_or(0, |f| f.bytes())
            + s.roles.as_ref().map_or(0, |r| r.bytes())
            + s.hand_size.as_ref().map_or(0, |r| r.bytes())
            + s.sba_batch.as_ref().map(|batch| batch.bytes()).unwrap_or(0)
            + s.entry_batch
                .as_ref()
                .map(|batch| batch.bytes())
                .unwrap_or(0)
            + s.last_target_library_move
                .as_ref()
                .map_or(0, |_| std::mem::size_of::<TargetLibraryMovement>())
            + s.known_hand.len() * std::mem::size_of::<ObjectRef>()
            + s.explore_data.as_ref().map_or(0, |d| d.bytes())
            + s.evidence_data.as_ref().map_or(0, |d| d.bytes())
            + (s.free_tokens.len()
                + s.static_sources.len()
                + s.cast_sources.len()
                + s.attack_sources.len()
                + s.tap_sources.len()
                + s.damage_sources.len()
                + s.discard_sources.len()
                + s.draw_sources.len()
                + s.cleanup_discards.len()
                + s.step_sources.len()
                + s.life_sources.len()
                + s.counter_sources.len()
                + s.death_sources.len()
                + s.attachment_sources.len())
                * std::mem::size_of::<CardId>()
            + s.players
                .iter()
                .map(|p| p.library.len() * std::mem::size_of::<CardId>())
                .sum::<usize>()
            + (s.stack.len() + s.pending_triggers.len() + s.ordering_triggers.len())
                * std::mem::size_of::<SpellView>()
            + s.resolving
                .as_ref()
                .map(|resolution| {
                    resolution.linked_copy.as_ref().map_or(0, |_| {
                        std::mem::size_of::<linked_copies::LinkedCopyCapture>()
                    }) + resolution.cascade.as_ref().map_or(0, |row| {
                        std::mem::size_of::<CascadeView>()
                            + row.exiled.len() * std::mem::size_of::<ObjectRef>()
                    }) + resolution
                        .explore
                        .as_ref()
                        .map_or(0, |_| std::mem::size_of::<ExploreChoiceView>())
                        + resolution
                            .immediate_blink_return
                            .as_ref()
                            .map_or(0, |rows| {
                                std::mem::size_of::<Vec<ReturnLinkView>>()
                                    + rows.len() * std::mem::size_of::<ReturnLinkView>()
                            })
                        + resolution
                            .milled_selection
                            .as_ref()
                            .map_or(0, |f| f.bytes())
                        + resolution
                            .optional_searches
                            .as_ref()
                            .map_or(0, |_| std::mem::size_of::<search::OptionalSearchSequence>())
                        + resolution.target_card_facts.as_ref().map_or(0, |values| {
                            std::mem::size_of::<Vec<programs::TargetCardFacts>>()
                                + values.len() * std::mem::size_of::<programs::TargetCardFacts>()
                        })
                        + resolution.target_library.as_ref().map_or(0, |f| f.bytes())
                        + resolution.forage.as_ref().map_or(0, |f| f.bytes())
                        + resolution.evidence.as_ref().map_or(0, |f| f.bytes())
                        + resolution.connive.as_ref().map_or(0, |c| c.bytes())
                        + resolution.mana_payment.as_ref().map_or(0, |_| {
                            std::mem::size_of::<resolution_mana::ResolutionPayment>()
                        })
                        + resolution.milled.len() * std::mem::size_of::<ObjectRef>()
                        + resolution
                            .choice
                            .as_ref()
                            .map(|choice| choice.bytes())
                            .unwrap_or(0)
                        + resolution
                            .search
                            .as_ref()
                            .map(|choice| {
                                std::mem::size_of::<search::SearchChoice>()
                                    + (choice.selected.len()
                                        + choice.battlefield.as_ref().map_or(0, Vec::len))
                                        * std::mem::size_of::<CardId>()
                            })
                            .unwrap_or(0)
                        + resolution
                            .library
                            .as_ref()
                            .map(|choice| {
                                (choice.remaining.len()
                                    + choice.top.len()
                                    + choice.bottom.len()
                                    + choice.graveyard.len())
                                    * std::mem::size_of::<CardId>()
                            })
                            .unwrap_or(0)
                        + resolution.selection.as_ref().map_or(0, |f| f.bytes())
                        + resolution
                            .return_attachment
                            .as_ref()
                            .map_or(0, |frame| frame.owned_bytes())
                        + resolution
                            .selected_discards
                            .iter()
                            .map(Vec::len)
                            .sum::<usize>()
                            * std::mem::size_of::<CardId>()
                })
                .unwrap_or(0)
            + s.announcing.as_ref().map_or(0, |a| {
                casualty::bytes(&a.casualty) + a.cast_costs.as_ref().map_or(0, |c| c.bytes())
            })
            + s.announcing
                .as_ref()
                .and_then(|a| a.convoke.as_ref())
                .map_or(0, |f| f.bytes())
            + s.announcing
                .as_ref()
                .and_then(|a| a.evidence.as_ref())
                .map_or(0, |e| e.bytes())
            + s.announcing
                .as_ref()
                .and_then(|a| a.forage.as_ref())
                .map_or(0, |f| f.bytes())
            + s.announcing
                .as_ref()
                .and_then(|a| a.additional_group.as_ref())
                .map_or(0, |v| {
                    std::mem::size_of::<Vec<ObjectRef>>()
                        + v.len() * std::mem::size_of::<ObjectRef>()
                })
            + s.announcing
                .as_ref()
                .and_then(|a| a.creature_counter_cost.as_ref())
                .map_or(0, |v| {
                    std::mem::size_of::<Vec<(ObjectRef, CounterKind)>>()
                        + v.len() * std::mem::size_of::<(ObjectRef, CounterKind)>()
                })
            + s.stack
                .iter()
                .chain(&s.pending_triggers)
                .chain(&s.ordering_triggers)
                .chain(s.resolving.iter().map(|resolution| &resolution.item))
                .chain(s.announcing.iter().map(|announcement| &announcement.item))
                .map(SpellView::owned_bytes)
                .sum::<usize>()
            + s.attackers.len() * std::mem::size_of::<Attacker>()
            + s.blocking.len() * std::mem::size_of::<CardId>()
            + s.blocks.len() * std::mem::size_of::<(CardId, CardId)>()
            + s.selected_attackers.len() * std::mem::size_of::<Attacker>()
            + s.selected_blocks.len() * std::mem::size_of::<(CardId, CardId)>()
            + s.hits.len() * std::mem::size_of::<DamageEvent>()
            + s.creature_damage_history.as_ref().map_or(0, |rows| {
                std::mem::size_of::<Vec<CreatureDamageHistoryView>>()
                    + rows.len() * std::mem::size_of::<CreatureDamageHistoryView>()
            })
            + s.first_strikers.len() * std::mem::size_of::<CardId>()
            + s.damage_choices.len() * std::mem::size_of::<DamageChoice>()
            + s.damage_choices
                .iter()
                .map(|d| d.blockers.len() * std::mem::size_of::<CardId>())
                .sum::<usize>()
    }
}

#[path = "engine_equipment.rs"]
mod equipment;

#[path = "engine_entry_choices.rs"]
mod entry_choices;
pub use entry_choices::EntryChoiceView;

#[path = "engine_entries.rs"]
mod entries;
#[path = "engine_entry_tap.rs"]
mod entry_tap;
pub use entries::{
    AuraAttachmentView, AuraEntryView, EntryLifeSelection, EntryLifeView, EntryTimestampObjectView,
    EntryTimestampView,
};
pub use entry_tap::{EntryTapEffect, EntryTapKind, EntryTapView};

pub use resolution_mana::ResolutionPaymentView;

pub use discard_events::DiscardTriggerView;

pub use draw_events::DrawTriggerView;

pub use connive::{ConniveChoiceView, ConniverView};

#[path = "engine_emblems.rs"]
mod emblems;

#[path = "engine_granted_abilities.rs"]
mod granted_abilities;

#[path = "engine_death_exile.rs"]
mod death_exile;
pub use death_exile::ExileDeathReplacementView;

#[path = "engine_temporary_abilities.rs"]
mod temporary_abilities;
pub use temporary_abilities::TemporaryAbilityView;

#[path = "engine_suspect.rs"]
mod suspect;

#[path = "engine_zone_effects.rs"]
mod zone_effects;
pub use zone_effects::{NextTurnControlExpiry, TemporaryControlView};

impl Game {
    fn no_maximum_hand_size(&self, player: Player) -> bool {
        self.maximum_hand_size(player).is_none()
    }
    fn continuous_land_plays(&self, player: Player) -> u64 {
        self.state
            .static_sources
            .iter()
            .filter(|&&id| self.state.objects[id as usize].controller == player)
            .map(|&id| {
                self.state.objects[id as usize]
                    .card
                    .static_ability_groups()
                    .iter()
                    .flat_map(|g| g.fragments)
                    .filter_map(|f| match f {
                        crate::cards::StaticAbilityFragment::AdditionalLandPlays(n)
                            if self.continuous_fragment_available(id, |x| {
                                matches!(
                                    x,
                                    crate::cards::StaticAbilityFragment::AdditionalLandPlays(_)
                                )
                            }) =>
                        {
                            Some(u64::from(*n))
                        }
                        _ => None,
                    })
                    .sum::<u64>()
            })
            .sum()
    }
}

#[path = "engine_card_quantities.rs"]
mod card_quantities;
pub use card_quantities::SourceCounterQuantityView;
pub use event_objects::{CreatureComparisonFacts, EventObjectFacts, EventObjectView};
pub use source_quantities::SourceQuantityView;
#[path = "engine_cast_entry_counters.rs"]
mod cast_entry_counters;
#[path = "engine_event_objects.rs"]
mod event_objects;
#[cfg(test)]
#[path = "engine_msh_event_objects_tests.rs"]
mod msh_event_objects_tests;
#[cfg(test)]
#[path = "engine_msh_source_quantities_tests.rs"]
mod msh_source_quantities_tests;
#[path = "engine_source_quantities.rs"]
mod source_quantities;
pub use cast_entry_counters::CastEntryCounterView;
#[path = "engine_counter_placement_history.rs"]
mod counter_placement_history;
pub use counter_placement_history::CounterPlacementHistoryView;

pub use search::OptionalSearchSequence;
#[path = "engine_event_counter_core.rs"]
mod event_counter_core;
pub use event_counter_core::CreatureDamageHistoryView;

#[path = "engine_continuous_control.rs"]
mod continuous_control;

#[path = "engine_attachment_death.rs"]
mod attachment_death;

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
#[path = "engine_big_library_tests.rs"]
mod big_library_tests;
#[cfg(test)]
#[path = "engine_big_token_tests.rs"]
mod big_token_tests;
#[cfg(test)]
#[path = "engine_territory_forge_tests.rs"]
mod territory_forge_tests;

#[cfg(test)]
#[path = "engine_big_inherited_tests.rs"]
mod big_inherited_tests;

#[path = "engine_sos_graveyard.rs"]
mod sos_graveyard;
#[cfg(test)]
#[path = "engine_sos_green_colorless_tests.rs"]
mod sos_green_colorless_tests;

#[path = "engine_cast_rollback.rs"]
mod cast_rollback;

#[cfg(test)]
#[path = "engine_power_up_tests.rs"]
mod power_up_tests;

#[cfg(test)]
#[path = "engine_plan_threshold_tests.rs"]
mod plan_threshold_tests;

#[cfg(test)]
#[path = "engine_msh_existing_tests.rs"]
mod msh_existing_tests;

#[cfg(test)]
#[path = "engine_milled_recovery_tests.rs"]
mod milled_recovery_tests;

#[cfg(test)]
#[path = "engine_msh_power_up_cards_tests.rs"]
mod msh_power_up_cards_tests;

#[cfg(test)]
#[path = "engine_counter_history_tests.rs"]
mod counter_history_tests;

#[cfg(test)]
#[path = "engine_msh_forms_tests.rs"]
mod msh_forms_tests;

#[cfg(test)]
#[path = "engine_keyword_counter_tests.rs"]
mod keyword_counter_tests;

#[cfg(test)]
#[path = "engine_msh_inherited_tests.rs"]
mod msh_inherited_tests;

#[cfg(test)]
#[path = "engine_msh_placement_tests.rs"]
mod msh_placement_tests;

#[cfg(test)]
#[path = "engine_finality_tests.rs"]
mod finality_tests;

#[cfg(test)]
#[path = "engine_msh_four_existing_tests.rs"]
mod msh_four_existing_tests;

#[cfg(test)]
#[path = "engine_msh_combined_tests.rs"]
mod msh_combined_tests;

#[cfg(test)]
#[path = "engine_msh_coulson_tests.rs"]
mod msh_coulson_tests;

#[cfg(test)]
#[path = "engine_msh_counter_batch_tests.rs"]
mod msh_counter_batch_tests;

#[cfg(test)]
#[path = "engine_resolution_alternative_cost_tests.rs"]
mod resolution_alternative_cost_tests;

#[cfg(test)]
#[path = "engine_msh_fifth_combined_tests.rs"]
mod msh_fifth_combined_tests;

#[cfg(test)]
#[path = "engine_msh_hideout_falcon_tests.rs"]
mod msh_hideout_falcon_tests;

#[cfg(test)]
#[path = "engine_msh_sentry_alien_tests.rs"]
mod msh_sentry_alien_tests;

#[cfg(test)]
#[path = "engine_msh_mandatory_union_tests.rs"]
mod msh_mandatory_union_tests;

#[cfg(test)]
#[path = "engine_msh_punch_zola_tests.rs"]
mod msh_punch_zola_tests;

#[cfg(test)]
#[path = "engine_msh_tap_batch_tests.rs"]
mod msh_tap_batch_tests;

#[cfg(test)]
#[path = "engine_msh_sixth_combined_tests.rs"]
mod msh_sixth_combined_tests;

#[path = "engine_tap_history.rs"]
mod tap_history;

#[cfg(test)]
#[path = "engine_msh_first_tap_tests.rs"]
mod msh_first_tap_tests;

#[cfg(test)]
#[path = "engine_msh_marvell_tests.rs"]
mod msh_marvell_tests;

#[cfg(test)]
#[path = "engine_msh_stature_tests.rs"]
mod msh_stature_tests;

#[cfg(test)]
#[path = "engine_msh_skrull_tests.rs"]
mod msh_skrull_tests;

#[cfg(test)]
#[path = "engine_msh_thirst_tests.rs"]
mod msh_thirst_tests;

#[cfg(test)]
#[path = "engine_msh_seventh_combined_tests.rs"]
mod msh_seventh_combined_tests;

#[cfg(test)]
#[path = "engine_msh_titania_tests.rs"]
mod msh_titania_tests;

#[cfg(test)]
#[path = "engine_msh_thor_tests.rs"]
mod msh_thor_tests;

#[cfg(test)]
#[path = "engine_msh_eighth_combined_tests.rs"]
mod msh_eighth_combined_tests;

#[cfg(test)]
#[path = "engine_msh_animation_death_tests.rs"]
mod msh_animation_death_tests;

#[cfg(test)]
#[path = "engine_msh_under_siege_tests.rs"]
mod msh_under_siege_tests;

#[cfg(test)]
#[path = "engine_msh_doom_tests.rs"]
mod msh_doom_tests;

#[cfg(test)]
#[path = "engine_msh_ninth_combined_tests.rs"]
mod msh_ninth_combined_tests;

#[cfg(test)]
#[path = "engine_msh_baron_tests.rs"]
mod msh_baron_tests;

#[cfg(test)]
#[path = "engine_msh_vision_tests.rs"]
mod msh_vision_tests;

#[path = "engine_hand_size.rs"]
mod hand_size;
pub use hand_size::{HandSizeSourceView, HandSizeTimelineView};

#[cfg(test)]
#[path = "engine_msh_ten_rings_tests.rs"]
mod msh_ten_rings_tests;

#[path = "engine_cast_symbol_quantities.rs"]
mod cast_symbol_quantities;

#[cfg(test)]
#[path = "engine_msh_namor_tests.rs"]
mod msh_namor_tests;

#[cfg(test)]
#[path = "engine_msh_tenth_combined_tests.rs"]
mod msh_tenth_combined_tests;

#[cfg(test)]
#[path = "engine_msh_galactus_tests.rs"]
mod msh_galactus_tests;

#[cfg(test)]
#[path = "engine_msh_us_agent_tests.rs"]
mod msh_us_agent_tests;

#[path = "engine_library_top.rs"]
mod library_top;
pub use library_top::LibraryRevealEvent;
#[cfg(test)]
#[path = "engine_msh_feast_tests.rs"]
mod msh_feast_tests;
#[cfg(test)]
#[path = "engine_msh_improvise_tests.rs"]
mod msh_improvise_tests;
#[cfg(test)]
#[path = "engine_msh_library_top_tests.rs"]
mod msh_library_top_tests;

#[cfg(test)]
#[path = "engine_msh_eleventh_combined_tests.rs"]
mod msh_eleventh_combined_tests;

#[cfg(test)]
#[path = "engine_msh_recent_targets_tests.rs"]
mod msh_recent_targets_tests;

#[cfg(test)]
#[path = "engine_msh_reflexive_tap_haste_tests.rs"]
mod msh_reflexive_tap_haste_tests;

#[cfg(test)]
#[path = "engine_msh_turn_facts_tests.rs"]
mod msh_turn_facts_tests;

#[cfg(test)]
#[path = "engine_msh_land_origins_tests.rs"]
mod msh_land_origins_tests;

#[cfg(test)]
#[path = "engine_msh_twelfth_combined_tests.rs"]
mod msh_twelfth_combined_tests;

#[cfg(test)]
#[path = "engine_msh_red_grim_tests.rs"]
mod msh_red_grim_tests;

#[cfg(test)]
#[path = "engine_msh_baxter_raft_tests.rs"]
mod msh_baxter_raft_tests;

#[cfg(test)]
#[path = "engine_msh_hand_exile_tests.rs"]
mod msh_hand_exile_tests;

#[cfg(test)]
#[path = "engine_msh_temporary_copy_cards_tests.rs"]
mod msh_temporary_copy_cards_tests;

#[cfg(test)]
#[path = "engine_msh_thirteenth_combined_tests.rs"]
mod msh_thirteenth_combined_tests;

#[cfg(test)]
#[path = "engine_msh_copy_identity_regression_tests.rs"]
mod msh_copy_identity_regression_tests;

#[cfg(test)]
#[path = "engine_parity_library_core_tests.rs"]
mod parity_library_core_tests;

#[cfg(test)]
#[path = "engine_msh_hand_entry_core_tests.rs"]
mod msh_hand_entry_core_tests;

#[cfg(test)]
#[path = "engine_msh_counter_quantity_core_tests.rs"]
mod msh_counter_quantity_core_tests;

#[cfg(test)]
#[path = "engine_msh_incursion_justice_loki_core_tests.rs"]
mod msh_incursion_justice_loki_core_tests;

#[cfg(test)]
#[path = "engine_msh_broad_legal_games_tests.rs"]
mod msh_broad_legal_games_tests;

#[cfg(test)]
#[path = "engine_msh_counter_three_tests.rs"]
mod msh_counter_three_tests;

#[cfg(test)]
#[path = "engine_msh_incursion_justice_loki_cards_tests.rs"]
mod msh_incursion_justice_loki_cards_tests;

#[cfg(test)]
#[path = "engine_msh_origin_nick_cards_tests.rs"]
mod msh_origin_nick_cards_tests;

#[cfg(test)]
#[path = "engine_msh_parity_library_paid_tests.rs"]
mod msh_parity_library_paid_tests;

#[cfg(test)]
#[path = "engine_msh_fourteenth_combined_tests.rs"]
mod msh_fourteenth_combined_tests;

#[cfg(test)]
#[path = "engine_msh_turn_cast_grants_tests.rs"]
mod msh_turn_cast_grants_tests;

#[cfg(test)]
#[path = "engine_msh_ability_copy_core_tests.rs"]
mod msh_ability_copy_core_tests;

#[cfg(test)]
#[path = "engine_msh_broker_thrall_tests.rs"]
mod msh_broker_thrall_tests;

#[cfg(test)]
#[path = "engine_msh_super_adaptoid_tests.rs"]
mod msh_super_adaptoid_tests;

#[cfg(test)]
#[path = "engine_msh_damage_replacement_tests.rs"]
mod msh_damage_replacement_tests;

#[cfg(test)]
#[path = "engine_shield_harness_tests.rs"]
mod shield_harness_tests;

#[cfg(test)]
#[path = "engine_msh_restricted_creatures_tests.rs"]
mod msh_restricted_creatures_tests;

#[cfg(test)]
#[path = "engine_msh_kingpin_tests.rs"]
mod msh_kingpin_tests;

#[cfg(test)]
#[path = "engine_msh_armor_hulk_sagas_tests.rs"]
mod msh_armor_hulk_sagas_tests;

#[cfg(test)]
#[path = "engine_msh_graveyard_plans_tests.rs"]
mod msh_graveyard_plans_tests;

#[cfg(test)]
#[path = "engine_msh_leader_tests.rs"]
mod msh_leader_tests;

#[cfg(test)]
#[path = "engine_msh_hand_choices_tests.rs"]
mod msh_hand_choices_tests;

#[cfg(test)]
#[path = "engine_msh_widow_worlds_tests.rs"]
mod msh_widow_worlds_tests;

#[cfg(test)]
#[path = "engine_msh_source_linked_sting_tests.rs"]
mod msh_source_linked_sting_tests;

#[cfg(test)]
#[path = "engine_msh_civil_war_tests.rs"]
mod msh_civil_war_tests;

#[cfg(test)]
#[path = "engine_msh_serum_tests.rs"]
mod msh_serum_tests;

#[cfg(test)]
#[path = "engine_msh_armor_tony_tests.rs"]
mod msh_armor_tony_tests;

#[cfg(test)]
#[path = "engine_msh_bruce_kang_tests.rs"]
mod msh_bruce_kang_tests;

#[cfg(test)]
#[path = "engine_msh_cosmic_cube_tests.rs"]
mod msh_cosmic_cube_tests;

#[cfg(test)]
#[path = "engine_msh_vision_quest_tests.rs"]
mod msh_vision_quest_tests;

#[cfg(test)]
#[path = "engine_msh_hawkeye_wrecking_tests.rs"]
mod msh_hawkeye_wrecking_tests;

#[cfg(test)]
#[path = "engine_msh_cast_target_copy_tests.rs"]
mod msh_cast_target_copy_tests;

#[cfg(test)]
#[path = "engine_msh_storm_tests.rs"]
mod msh_storm_tests;

#[cfg(test)]
#[path = "engine_msh_entering_copy_tests.rs"]
mod msh_entering_copy_tests;

#[cfg(test)]
#[path = "engine_msh_photon_barrage_tests.rs"]
mod msh_photon_barrage_tests;

#[path = "engine_graveyard_symbol_cost.rs"]
mod graveyard_symbol_cost;

#[cfg(test)]
#[path = "engine_msh_zemo_tests.rs"]
mod msh_zemo_tests;

#[cfg(test)]
#[path = "engine_msh_final_shared_interactions_tests.rs"]
mod msh_final_shared_interactions_tests;

#[cfg(test)]
#[path = "engine_msh_final_copy_interactions_tests.rs"]
mod msh_final_copy_interactions_tests;

#[cfg(test)]
#[path = "engine_msh_controlled_policy_tests.rs"]
mod msh_controlled_policy_tests;
