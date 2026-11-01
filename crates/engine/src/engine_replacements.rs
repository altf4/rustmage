//! Owned, event-local replacement ordering. Immutable programs stay in the catalogue.
use super::*;
use crate::cards::{AnimationSpec, ReplacementFilter, ReplacementSpec};
use std::collections::VecDeque;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ResumeMode {
    Program,
    Pending,
    CastCosts,
    ActivationCosts,
    Entry,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ReplacementEffectView {
    pub source: ObjectRef,
    pub source_card: Card,
    pub ability: u16,
}
/// Actual intrinsic prevention effects, separate from printed replacement slots.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamagePreventionKind {
    ShieldCounter,
    CombatBySource,
    CombatToRecipient,
    ProtectionEverything,
    ProtectionInstantSorcery,
    SourceLinkedDamage,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct DamagePreventionView {
    pub source: ObjectRef,
    pub source_card: Card,
    pub kind: DamagePreventionKind,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct DamageEventOrderView {
    pub index: u32,
    pub event: DamageEvent,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Serialize)]
pub struct ReplacementChoiceView {
    pub kind: &'static str,
    pub object: Option<ObjectRef>,
    pub counters: Option<Counters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_counters: Option<PlayerCounterPlacementView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proliferate: Option<ProliferateReplacementView>,
    pub tokens: Vec<TokenGroupView>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub copy_recipes: Vec<Option<CopyRecipe>>,
    /// Parallel to tokens for Role events; None denotes an unbound replacement addition.
    /// Ordinary token and counter events use an empty vector.
    pub role_attachments: Vec<Option<ObjectRef>>,
    pub effects: Vec<ReplacementEffectView>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub self_counter_effects: Vec<SelfPlayerCounterReplacement>,
    /// Current public damage event, before the next chosen replacement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub damage: Option<DamageEvent>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub damage_preventions: Vec<DamagePreventionView>,
    /// Exact pending heals applied before the simultaneous final damage batch.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub heals_before_damage: Vec<ObjectRef>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub damage_event_order: Vec<DamageEventOrderView>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ProliferateReplacementView {
    pub player: Player,
    pub amount: u128,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PlayerCounterPlacementView {
    pub player: Player,
    pub kind: crate::cards::PlayerCounterKind,
    pub amount: u64,
    pub placed_by: Player,
    /// False for a cost event, including after amount replacements.
    pub effect: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CounterPlacement {
    pub object: ObjectRef,
    pub placed_by: Player,
    pub card: Card,
    pub owner: Player,
    pub controller: Player,
    pub protector: Option<Player>,
    pub subtypes: GrantedSubtypes,
    pub kind: u16,
    pub power: i128,
    pub amount: Counters,
    pub effect: bool,
    pub entry_index: Option<usize>,
    pub private: bool,
}
/// A replacement owned by the exact resolving effect that produced its event.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SelfPlayerCounterReplacement {
    pub origin_stack_id: u64,
    pub effect: u16,
    pub player: Player,
    pub kind: crate::cards::PlayerCounterKind,
    pub factor: u32,
    pub add: u32,
    pub redirect_player: Option<Player>,
    pub redirect_kind: Option<crate::cards::PlayerCounterKind>,
    pub split: &'static [crate::cards::PlayerCounterSplit],
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct CounterBatch {
    // Player poison additions commit with the permanent placements after all
    // permanent replacement choices have completed.
    player_events: player_counter_events::PlayerCounterEvents,
    self_effects: Vec<SelfPlayerCounterReplacement>,
    player_placed_by: Option<Player>,
    player_effect: [bool; 2],
    next_player: usize,
    proliferated: Option<Player>,
    completion: Option<Box<CounterReflexiveCompletion>>,
    placements: Vec<CounterPlacement>,
    next: usize,
    used: Vec<(ObjectRef, u16)>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn apply_own_player_counter_replacement(
    batch: &mut CounterBatch,
    effect: SelfPlayerCounterReplacement,
    active: Player,
) {
    let event = &batch.player_events.events()[batch.next_player];
    let event_id = event.id;
    let player = event.player;
    let kind = event.kind;
    let amount = event.amount;
    let scale = |factor: u32, add: u32| {
        amount
            .checked_mul(u64::from(factor))
            .and_then(|n| n.checked_add(u64::from(add)))
            .capacity("self counter replacement exhausted")
    };
    let outcomes: Vec<_> = if effect.split.is_empty() {
        vec![player_counter_events::CounterDestination {
            player: effect.redirect_player.unwrap_or(player),
            kind: effect.redirect_kind.unwrap_or(kind),
            amount: scale(effect.factor, effect.add),
        }]
    } else {
        effect
            .split
            .iter()
            .map(|out| player_counter_events::CounterDestination {
                player: if out.to_opponent {
                    player.other()
                } else {
                    player
                },
                kind: out.kind.unwrap_or(kind),
                amount: scale(out.factor, out.add),
            })
            .collect()
    };
    assert!(batch.player_events.apply_self(
        event_id,
        player_counter_events::EffectIdentity {
            origin_stack_id: effect.origin_stack_id,
            index: effect.effect
        },
        &outcomes
    ));
    batch
        .player_events
        .prioritize_active_from(batch.next_player, active);
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn current_player_counter(
    batch: &CounterBatch,
    active: Player,
) -> Option<(Player, crate::cards::PlayerCounterKind)> {
    let event = batch.player_events.events().get(batch.next_player)?;
    if batch.next < batch.placements.len()
        && !(event.player == active && batch.placements[batch.next].controller != active)
    {
        return None;
    }
    Some((event.player, event.kind))
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct CounterReflexiveCompletion {
    item: SpellView,
    kind: CounterKind,
    ability: u16,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct TokenGroupView {
    pub token: Card,
    pub amount: u128,
    pub tapped: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RoleReflexiveCompletion {
    pub recipient: ObjectRef,
    pub token: Card,
    pub item: SpellView,
    pub ability: u16,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum TokenCompletion {
    None,
    CounterOtherWizards,
    Counters {
        kind: CounterKind,
        amount: u64,
    },
    CounterSet(Box<Counters>),
    /// Evaluate the controller's graveyard after this creation event completes.
    GraveyardGroups(std::num::NonZeroU16),
    TemporaryCopy(Box<SpellView>),
    AnimateNoncreatures(AnimationSpec),
    Keywords {
        source: ObjectRef,
        keywords: Keywords,
    },
    Reflexive {
        item: Box<SpellView>,
        ability: u16,
    },
    ReflexiveCreated {
        item: Box<SpellView>,
        ability: u16,
    },
    JobAttach(ObjectRef),
    /// Creation precedes attachment; an absent/illegal recipient leaves tokens intact.
    AttachCreatedTo(ObjectRef),
    RoleAttach(ObjectRef),
    RoleReflexive(Box<RoleReflexiveCompletion>),
    /// One mass creation event; original group indices retain exact recipients.
    RoleAttachMany(Vec<ObjectRef>),
    Mobilize,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct TokenBatch {
    recipes: Vec<Option<CopyRecipe>>,
    player: Player,
    groups: Vec<TokenGroupView>,
    effect: bool,
    used: Vec<(ObjectRef, u16)>,
    completion: TokenCompletion,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl TokenBatch {
    fn artifact_at(&self, index: usize) -> bool {
        self.recipes
            .get(index)
            .and_then(|recipe| *recipe)
            .map_or_else(
                || self.groups[index].token.is_artifact(),
                |recipe| recipe.kinds() & 16 != 0,
            )
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct DamageBatch {
    events: Vec<DamageEvent>,
    heals: Vec<ObjectRef>,
    used_preventions: Vec<(ObjectRef, DamagePreventionKind)>,
    ordered_event: bool,
    next: usize,
    used: Vec<(ObjectRef, u16)>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct ProliferateBatch {
    player: Player,
    amount: u128,
    used: Vec<(ObjectRef, u16)>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
enum Event {
    Proliferate(ProliferateBatch),
    Damage(DamageBatch),
    Counters(CounterBatch),
    Tokens(TokenBatch),
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
enum Work {
    AmassSubtype {
        source: ObjectRef,
        recipient: ObjectRef,
        attach: bool,
    },
    AnimateCreated {
        created: Vec<ObjectRef>,
        spec: AnimationSpec,
    },
    CreatedCounters {
        player: Player,
        created: Vec<ObjectRef>,
        counters: Box<Counters>,
    },
    CreatedGraveyardCounters {
        player: Player,
        created: Vec<ObjectRef>,
        group: std::num::NonZeroU16,
    },
    Damage(DamageBatch),
    Counters(CounterBatch),
    Tokens(TokenBatch),
    Proliferate(ProliferateBatch),
    DestroyAndCreate {
        targets: Vec<BoundTarget>,
        token: Card,
        amount: u16,
    },
    EmptyExplore {
        explorer: ExplorerView,
        counter_added: bool,
    },
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReplacementState {
    work: VecDeque<Work>,
    active: Option<Event>,
    attachment: Option<(ObjectRef, Vec<ObjectRef>)>,
    resume: Option<Pending>,
    mode: ResumeMode,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ReplacementState {
    pub fn bytes(&self) -> usize {
        let counter_bytes = |b: &CounterBatch| {
            b.player_events.owned_bytes()
                + b.self_effects.len() * std::mem::size_of::<SelfPlayerCounterReplacement>()
                + b.placements.len() * std::mem::size_of::<CounterPlacement>()
                + b.placements
                    .iter()
                    .map(|p| p.amount.owned_bytes())
                    .sum::<usize>()
                + b.used.len() * std::mem::size_of::<(ObjectRef, u16)>()
                + b.completion.as_ref().map_or(0, |c| {
                    std::mem::size_of::<CounterReflexiveCompletion>() + c.item.owned_bytes()
                })
        };
        let token_bytes = |b: &TokenBatch| {
            b.groups.len() * std::mem::size_of::<TokenGroupView>()
                + b.recipes.len() * std::mem::size_of::<Option<CopyRecipe>>()
                + b.used.len() * std::mem::size_of::<(ObjectRef, u16)>()
                + match &b.completion {
                    TokenCompletion::CounterSet(counters) => {
                        std::mem::size_of::<Counters>() + counters.owned_bytes()
                    }
                    TokenCompletion::TemporaryCopy(item)
                    | TokenCompletion::Reflexive { item, .. }
                    | TokenCompletion::ReflexiveCreated { item, .. } => {
                        std::mem::size_of::<SpellView>() + item.owned_bytes()
                    }
                    TokenCompletion::RoleReflexive(c) => {
                        std::mem::size_of::<RoleReflexiveCompletion>() + c.item.owned_bytes()
                    }
                    TokenCompletion::RoleAttachMany(recipients) => {
                        recipients.len() * std::mem::size_of::<ObjectRef>()
                    }
                    _ => 0,
                }
        };
        std::mem::size_of::<Self>()
            + self.work.len() * std::mem::size_of::<Work>()
            + self
                .work
                .iter()
                .map(|w| match w {
                    Work::AnimateCreated { created, .. } => {
                        created.len() * std::mem::size_of::<ObjectRef>()
                    }
                    Work::Damage(b) => {
                        b.events.len() * std::mem::size_of::<DamageEvent>()
                            + b.heals.len() * std::mem::size_of::<ObjectRef>()
                            + b.used_preventions.len()
                                * std::mem::size_of::<(ObjectRef, DamagePreventionKind)>()
                            + b.used.len() * std::mem::size_of::<(ObjectRef, u16)>()
                    }
                    Work::Proliferate(b) => b.used.len() * std::mem::size_of::<(ObjectRef, u16)>(),
                    Work::Counters(b) => counter_bytes(b),
                    Work::Tokens(b) => token_bytes(b),
                    Work::CreatedGraveyardCounters { created, .. } => {
                        created.len() * std::mem::size_of::<ObjectRef>()
                    }
                    Work::CreatedCounters {
                        created, counters, ..
                    } => {
                        created.len() * std::mem::size_of::<ObjectRef>()
                            + std::mem::size_of::<Counters>()
                            + counters.owned_bytes()
                    }
                    Work::DestroyAndCreate { targets, .. } => {
                        targets.len() * std::mem::size_of::<BoundTarget>()
                    }
                    _ => 0,
                })
                .sum::<usize>()
            + self.active.as_ref().map_or(0, |b| {
                // The inline Option<Event> storage is already in size_of::<Self>().
                match b {
                    Event::Proliferate(b) => b.used.len() * std::mem::size_of::<(ObjectRef, u16)>(),
                    Event::Damage(b) => {
                        b.events.len() * std::mem::size_of::<DamageEvent>()
                            + b.heals.len() * std::mem::size_of::<ObjectRef>()
                            + b.used_preventions.len()
                                * std::mem::size_of::<(ObjectRef, DamagePreventionKind)>()
                            + b.used.len() * std::mem::size_of::<(ObjectRef, u16)>()
                    }
                    Event::Counters(b) => counter_bytes(b),
                    Event::Tokens(b) => token_bytes(b),
                }
            })
            + self
                .attachment
                .as_ref()
                .map_or(0, |(_, r)| r.len() * std::mem::size_of::<ObjectRef>())
    }
}
/// Fast-path guard: project an entrant only when a power-sensitive counter
/// replacement could apply, including its own as-entry replacement.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn entry_power_sensitive<W: super::ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    controller: Player,
    incoming: Card,
) -> bool {
    let has_filter = |spec: &ReplacementSpec| {
        matches!(
            spec,
            ReplacementSpec::Counters {
                filter: ReplacementFilter::CreaturePowerAtMost(_),
                ..
            }
        )
    };
    incoming.replacement_effects().iter().any(has_filter)
        || world.static_sources().iter().any(|&id| {
            let object = world.object(id);
            object.zone == Zone::Battlefield && object.controller == controller
                && object.card.replacement_effects().iter().enumerate().any(|(index, spec)| {
                    has_filter(spec) && world.fragment_available(id, |fragment| matches!(fragment,
                        crate::cards::StaticAbilityFragment::Replacement(slot, _) if usize::from(*slot) == index))
                })
        })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn global_counters_prohibited<W: super::ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    kinds: u16,
) -> bool {
    use crate::cards::StaticAbilityFragment;
    world.static_sources().iter().any(|&id| {
        let source = world.object(id);
        source.zone == Zone::Battlefield
            && source.card.static_ability_groups().iter().any(|g| g.fragments.iter().any(|f| matches!(f, StaticAbilityFragment::CounterProhibition(mask) if *mask & kinds != 0)))
            && world.fragment_available(id, |f| matches!(f, StaticAbilityFragment::CounterProhibition(mask) if *mask & kinds != 0))
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn counters_prohibited<W: super::ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    object: ObjectRef,
) -> bool {
    use crate::cards::StaticAbilityFragment;
    let target = world.object(object.card);
    if target.zone != Zone::Battlefield || world.reference(object.card) != object {
        return false;
    }
    let attached = world.attachment_sources().iter().any(|&id| {
        let source = world.object(id);
        source.zone == Zone::Battlefield
            && source.attached_to == Some(object)
            && source.card.static_ability_groups().iter().any(|g| {
                g.fragments
                    .iter()
                    .any(|f| matches!(f, StaticAbilityFragment::AttachedCounterProhibition))
            })
            && world.fragment_available(id, |f| {
                matches!(f, StaticAbilityFragment::AttachedCounterProhibition)
            })
    });
    attached || global_counters_prohibited(world, world.kinds(object.card))
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn placement_prohibited<W: super::ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    p: &CounterPlacement,
) -> bool {
    if p.entry_index.is_some() {
        global_counters_prohibited(world, p.kind)
    } else {
        counters_prohibited(world, p.object)
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn matches_filter(filter: ReplacementFilter, p: &CounterPlacement) -> bool {
    let subtype = |s| p.subtypes.has(p.card, s);
    match filter {
        ReplacementFilter::CreaturePowerAtMost(maximum) => p.kind & 1 != 0 && p.power <= maximum,
        ReplacementFilter::Permanent => true,
        ReplacementFilter::BattleProtectedByOpponent => {
            p.kind & crate::cards::BATTLE_KIND != 0 && p.protector == Some(p.controller.other())
        }
        ReplacementFilter::Creature => p.kind & 1 != 0,
        ReplacementFilter::CreatureOrVehicle => p.kind & 1 != 0 || subtype("Vehicle"),
        ReplacementFilter::CreatureSpacecraftPlanet => {
            p.kind & 1 != 0 || subtype("Spacecraft") || subtype("Planet")
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn counter_applies(spec: ReplacementSpec, p: &CounterPlacement) -> bool {
    let (filter, kind, effects_only) = match spec {
        ReplacementSpec::Counters {
            filter,
            kind,
            effects_only,
            ..
        } => (filter, kind, effects_only),
        ReplacementSpec::CountersPlacedByController if p.placed_by == p.controller => {
            (ReplacementFilter::Permanent, None, false)
        }
        _ => return false,
    };
    matches_filter(filter, p)
        && (!effects_only || p.effect)
        && kind.map_or_else(|| p.amount.has_any(), |k| p.amount.get(k) > 0)
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn modify_counter(spec: ReplacementSpec, p: &mut CounterPlacement) {
    let (kind, factor, add) = match spec {
        ReplacementSpec::Counters {
            kind, factor, add, ..
        } => (kind, factor, add),
        ReplacementSpec::CountersPlacedByController => (None, 1, 1),
        _ => unreachable!(),
    };
    let mut result = Counters::default();
    for k in CounterKind::ALL {
        let n = p.amount.get(k);
        let n = if n > 0 && kind.is_none_or(|only| only == k) {
            n.checked_mul(u64::from(factor))
                .and_then(|n| n.checked_add(u64::from(add)))
                .capacity("replacement counter quantity exhausted")
        } else {
            n
        };
        result.add(k, n);
    }
    p.amount = result;
    // CR614.16: a replacement effect itself can make a non-effect event eligible.
    p.effect = true;
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn counter_replacements<W: super::ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    p: &CounterPlacement,
    used: &[(ObjectRef, u16)],
) -> Vec<ReplacementEffectView> {
    let mut out = Vec::new();
    let mut add = |reference: ObjectRef, card: Card, controller: Player| {
        if controller != p.controller {
            return;
        }
        for (index, &spec) in card.replacement_effects().iter().enumerate() {
            if world.object(reference.card).zone == Zone::Battlefield && world.reference(reference.card) == reference
                    && !world.fragment_available(reference.card, |fragment| matches!(fragment,
                        crate::cards::StaticAbilityFragment::Replacement(slot, _) if usize::from(*slot) == index)) { continue; }
            let ability = u16::try_from(index).expect("immutable replacement index");
            if !used.contains(&(reference, ability)) && counter_applies(spec, p) {
                out.push(ReplacementEffectView {
                    source: reference,
                    source_card: card,
                    ability,
                });
            }
        }
    };
    for &id in world.static_sources() {
        let o = world.object(id);
        if o.zone == Zone::Battlefield {
            add(world.reference(id), o.card, o.controller);
        }
    }
    // The counter replacement variants above affect a general set of
    // permanents. Under CR 614.12 they cannot apply from the incoming
    // permanent itself; self-only entry effects are handled by the entry plan.
    out
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn modify_proliferate(
    spec: ReplacementSpec,
    batch: &mut ProliferateBatch,
    source: ObjectRef,
    ability: u16,
) -> Option<proliferate::ProliferateGroup> {
    let ReplacementSpec::Proliferate { factor, add, .. } = spec else {
        unreachable!()
    };
    let created = u128::from(factor)
        .checked_add(u128::from(add))
        .capacity("proliferate quantity exhausted");
    batch.used.push((source, ability));
    batch.amount = u128::from(created != 0);
    (created > 1).then(|| proliferate::ProliferateGroup {
        count: created - 1,
        used: batch.used.clone(),
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    fn proliferate_replacements(&self, batch: &ProliferateBatch) -> Vec<ReplacementEffectView> {
        if batch.amount == 0 {
            return Vec::new();
        }
        let world = self.characteristic_query();
        let mut out = Vec::new();
        for &id in &self.state.static_sources {
            let object = &self.state.objects[id as usize];
            if object.zone != Zone::Battlefield {
                continue;
            }
            let reference = self.object_ref(id);
            for (index, spec) in object.card.replacement_effects().iter().enumerate() {
                let ReplacementSpec::Proliferate { relation, .. } = spec else {
                    continue;
                };
                let related = match relation {
                    CastRelation::You => batch.player == object.controller,
                    CastRelation::Opponent => batch.player != object.controller,
                    CastRelation::Any => true,
                };
                let ability = u16::try_from(index).expect("replacement index");
                if related
                    && !batch.used.contains(&(reference, ability))
                    && ability_world::CharacteristicWorld::fragment_available(
                        &world,
                        id,
                        |fragment| matches!(fragment, crate::cards::StaticAbilityFragment::Replacement(slot, _) if *slot == ability),
                    )
                {
                    out.push(ReplacementEffectView {
                        source: reference,
                        source_card: object.card,
                        ability,
                    });
                }
            }
        }
        out
    }
    pub(super) fn stage_proliferate_replacements(
        &mut self,
        player: Player,
        used: Vec<(ObjectRef, u16)>,
    ) {
        let batch = ProliferateBatch {
            player,
            amount: 1,
            used,
        };
        if self.state.replacements.is_none() && self.proliferate_replacements(&batch).is_empty() {
            self.state
                .resolving
                .as_mut()
                .unwrap()
                .proliferate
                .as_mut()
                .unwrap()
                .ready = true;
            return;
        }
        let state = self.state.replacements.get_or_insert_with(|| {
            Box::new(ReplacementState {
                work: VecDeque::new(),
                active: None,
                attachment: None,
                resume: None,
                mode: ResumeMode::Program,
            })
        });
        assert_eq!(state.mode, ResumeMode::Program);
        state.work.push_back(Work::Proliferate(batch));
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn modified_player_counters(spec: ReplacementSpec, amount: u64) -> u64 {
    let ReplacementSpec::PlayerCounters { factor, add, .. } = spec else {
        unreachable!()
    };
    amount
        .checked_mul(u64::from(factor))
        .and_then(|n| n.checked_add(u64::from(add)))
        .capacity("player counter replacement quantity exhausted")
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn player_counter_rule_matches(
    spec: ReplacementSpec,
    controller: Player,
    player: Player,
    placed_by: Player,
    effect: bool,
    actual_kind: crate::cards::PlayerCounterKind,
) -> bool {
    let ReplacementSpec::PlayerCounters {
        kind,
        relation,
        placed_by: placed_relation,
        effects_only,
        ..
    } = spec
    else {
        return false;
    };
    if kind != actual_kind {
        return false;
    }
    let matches = |r| match r {
        CastRelation::You => player == controller,
        CastRelation::Opponent => player != controller,
        CastRelation::Any => true,
    };
    let placer_matches = placed_relation.is_none_or(|r| match r {
        CastRelation::You => placed_by == controller,
        CastRelation::Opponent => placed_by != controller,
        CastRelation::Any => true,
    });
    matches(relation) && placer_matches && (!effects_only || effect)
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    // CR 614.17b-c: a prohibited event cannot be paid as a cost and
    // ordinary replacement effects cannot replace the prohibited event.
    pub(super) fn player_counters_prohibited(
        &self,
        player: Player,
        kind: crate::cards::PlayerCounterKind,
    ) -> bool {
        let world = self.characteristic_query();
        self.state.static_sources.iter().any(|&id| {
            let source = &self.state.objects[id as usize];
            source.zone == Zone::Battlefield && source.card.static_ability_groups().iter().any(|group|
                group.fragments.iter().any(|fragment| {
                    let crate::cards::StaticAbilityFragment::PlayerCounterProhibition { kind: k, relation } = fragment else { return false; };
                    if *k != kind { return false; }
                    let related = match relation {
                        CastRelation::You => player == source.controller,
                        CastRelation::Opponent => player != source.controller,
                        CastRelation::Any => true,
                    };
                    related && ability_world::CharacteristicWorld::fragment_available(&world, id,
                        |f| matches!(f, crate::cards::StaticAbilityFragment::PlayerCounterProhibition { kind: candidate, relation: r } if candidate == k && r == relation))
                }))
        })
    }
    fn player_counter_replacements(
        &self,
        player: Player,
        kind: crate::cards::PlayerCounterKind,
        placed_by: Player,
        effect: bool,
        amount: u64,
        used: &[(ObjectRef, u16)],
    ) -> Vec<ReplacementEffectView> {
        if amount == 0 {
            return Vec::new();
        }
        let world = self.characteristic_query();
        let mut out = Vec::new();
        for &id in &self.state.static_sources {
            let object = &self.state.objects[id as usize];
            if object.zone != Zone::Battlefield {
                continue;
            }
            let reference = self.object_ref(id);
            for (index, spec) in object.card.replacement_effects().iter().enumerate() {
                let ability = u16::try_from(index).expect("replacement index");
                if player_counter_rule_matches(
                    *spec,
                    object.controller,
                    player,
                    placed_by,
                    effect,
                    kind,
                ) && !used.contains(&(reference, ability))
                    && ability_world::CharacteristicWorld::fragment_available(
                        &world,
                        id,
                        |fragment| matches!(fragment, crate::cards::StaticAbilityFragment::Replacement(slot, _) if *slot == ability),
                    )
                {
                    out.push(ReplacementEffectView {
                        source: reference,
                        source_card: object.card,
                        ability,
                    });
                }
            }
        }
        out
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn counter_outcomes<W: super::ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    placement: &CounterPlacement,
    used: &mut Vec<(ObjectRef, u16)>,
    refresh: &dyn Fn(&CounterPlacement) -> CounterPlacement,
    emit: &mut dyn FnMut(&Counters) -> bool,
) -> bool {
    let placement = refresh(placement);
    if placement_prohibited(world, &placement) {
        return emit(&Counters::default());
    }
    let choices = counter_replacements(world, &placement, used);
    if choices.is_empty() {
        return emit(&placement.amount);
    }
    for choice in choices {
        let mut next = placement.clone();
        modify_counter(
            choice.source_card.replacement_effects()[usize::from(choice.ability)],
            &mut next,
        );
        used.push((choice.source, choice.ability));
        if counter_outcomes(world, &next, used, refresh, emit) {
            return true;
        }
        used.pop();
    }
    false
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone)]
pub(super) struct CounterPreview {
    pub index: usize,
    pub placement: CounterPlacement,
    pub used: Vec<(ObjectRef, u16)>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn projected_entry_counter(
        &self,
        choice: Option<(ObjectRef, u16)>,
    ) -> Option<CounterPreview> {
        let Event::Counters(batch) = self.state.replacements.as_ref()?.active.as_ref()? else {
            return None;
        };
        let mut placement = batch.placements[batch.next].clone();
        let index = placement.entry_index?;
        let mut used = batch.used.clone();
        if let Some((source, ability)) = choice {
            let effect = counter_replacements(self, &placement, &used)
                .into_iter()
                .find(|e| e.source == source && e.ability == ability)?;
            modify_counter(
                effect.source_card.replacement_effects()[usize::from(ability)],
                &mut placement,
            );
            used.push((source, ability));
        }
        Some(CounterPreview {
            index,
            placement,
            used,
        })
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn active_rule_fragment(
        &self,
        id: CardId,
        predicate: impl Fn(&crate::cards::StaticAbilityFragment) -> bool,
    ) -> bool {
        self.on_battlefield(id)
            && self.state.objects[id as usize]
                .card
                .static_ability_groups()
                .iter()
                .any(|g| g.fragments.iter().any(&predicate))
            && self.continuous_fragment_available(id, predicate)
    }
    pub(super) fn player_has_hexproof(&self, player: Player) -> bool {
        self.state.players[player.index()].hexproof_until_end_turn
            || self.state.static_sources.iter().any(|&id| {
                self.state.objects[id as usize].controller == player
                    && self.active_rule_fragment(id, |f| match *f {
                        crate::cards::StaticAbilityFragment::PlayerHexproof => true,
                        crate::cards::StaticAbilityFragment::PlayerHexproofIfSourceCounter {
                            kind,
                            minimum,
                        } => self.counter_count(id, kind) >= minimum,
                        _ => false,
                    })
            })
    }
    pub(super) fn controlled_spells_uncounterable(&self, player: Player, spell: Card) -> bool {
        self.state.static_sources.iter().any(|&id| self.state.objects[id as usize].controller == player && self.active_rule_fragment(id, |f| matches!(f, crate::cards::StaticAbilityFragment::ControlledSpellsUncounterable(kinds) if spell.kind_mask() & kinds != 0)))
    }
    fn damage_recipient_controller(&self, event: &DamageEvent) -> Player {
        match event.target {
            Target::Player(p) => p,
            Target::Card(id) => self.state.objects[id as usize].controller,
            _ => unreachable!(),
        }
    }
    fn damage_replacements(
        &self,
        event: &DamageEvent,
        used: &[(ObjectRef, u16)],
    ) -> Vec<ReplacementEffectView> {
        if event.amount <= 0 {
            return Vec::new();
        }
        let mut out = Vec::new();
        for &id in &self.state.static_sources {
            if !self.on_battlefield(id) {
                continue;
            }
            let object = &self.state.objects[id as usize];
            let source = self.object_ref(id);
            for (index, &spec) in object.card.replacement_effects().iter().enumerate() {
                let ability = u16::try_from(index).expect("immutable replacement index");
                if used.contains(&(source, ability)) || !self.continuous_fragment_available(id, |f| matches!(f, crate::cards::StaticAbilityFragment::Replacement(slot, _) if usize::from(*slot) == index)) { continue; }
                let applies = match spec {
                    ReplacementSpec::DoubleCreatureDamage => {
                        event.source.facts.battlefield
                            && event.source.facts.is_creature()
                            && event.source.controller == object.controller
                    }
                    ReplacementSpec::DoubleOpponentDamage => {
                        event.source.controller == object.controller
                            && self.damage_recipient_controller(event) != object.controller
                    }
                    ReplacementSpec::NoncombatDamagePlusSourcePower
                    | ReplacementSpec::NoncombatDamageToOpponentPlus(_) => {
                        !event.combat
                            && event.source.controller == object.controller
                            && self.damage_recipient_controller(event) != object.controller
                    }
                    ReplacementSpec::DoubleEquippedDamage => {
                        object.attached_to.is_some_and(|recipient| {
                            recipient == event.source.reference
                                && self.on_battlefield(recipient.card)
                                && self.object_ref(recipient.card) == recipient
                                && self.is_creature(recipient.card)
                        })
                    }
                    ReplacementSpec::HealPreviousDamageToSelf => {
                        matches!(event.target, Target::Card(target) if self.on_battlefield(target) && self.object_ref(target) == source)
                    }
                    ReplacementSpec::PreventAllDamageToSelf => {
                        event.preventable
                            && !self.damage_prevention_prohibited()
                            && matches!(event.target, Target::Card(target) if self.on_battlefield(target) && self.object_ref(target) == source)
                    }
                    ReplacementSpec::PreventOtherCreatureNoncombatDamage => {
                        event.preventable
                            && !self.damage_prevention_prohibited()
                            && !event.combat
                            && matches!(event.target, Target::Card(target) if self.on_battlefield(target) && self.is_creature(target) && self.object_ref(target) != source && self.state.objects[target as usize].controller == object.controller)
                    }
                    _ => false,
                };
                if applies {
                    out.push(ReplacementEffectView {
                        source,
                        source_card: object.card,
                        ability,
                    });
                }
            }
        }
        out
    }
    fn damage_preventions(
        &self,
        event: &DamageEvent,
        used: &[(ObjectRef, DamagePreventionKind)],
    ) -> Vec<DamagePreventionView> {
        if event.amount <= 0 {
            return Vec::new();
        }
        let mut out = Vec::new();
        if let Target::Card(id) = event.target
            && self.counter_count(id, CounterKind::Shield) > 0
            && !used.contains(&(self.object_ref(id), DamagePreventionKind::ShieldCounter))
        {
            out.push(DamagePreventionView {
                source: self.object_ref(id),
                source_card: self.state.objects[id as usize].card,
                kind: DamagePreventionKind::ShieldCounter,
            });
        }
        if !event.preventable || self.damage_prevention_prohibited() {
            return out;
        }
        if let Some(roles) = &self.state.roles {
            for row in &roles.source_linked_effects {
                if row.kind == SourceLinkedEffectKind::PreventDamageBy
                    && source_linked_effects::active(self, row)
                    && row.recipient == event.source.reference
                    && !used.contains(&(row.source, DamagePreventionKind::SourceLinkedDamage))
                {
                    out.push(DamagePreventionView {
                        source: row.source,
                        source_card: self.state.objects[row.source.card as usize].card,
                        kind: DamagePreventionKind::SourceLinkedDamage,
                    });
                }
            }
        }
        if event.combat && event.source.keywords.contains(Keywords::PREVENT_COMBAT_BY) {
            out.push(DamagePreventionView {
                source: event.source.reference,
                source_card: event.source.facts.card,
                kind: DamagePreventionKind::CombatBySource,
            });
        }
        if let Target::Card(id) = event.target {
            let keywords = self.keywords(id);
            let mut add = |kind| {
                out.push(DamagePreventionView {
                    source: self.object_ref(id),
                    source_card: self.state.objects[id as usize].card,
                    kind,
                })
            };
            if event.combat && keywords.contains(Keywords::PREVENT_COMBAT_TO) {
                add(DamagePreventionKind::CombatToRecipient);
            }
            if keywords.contains(Keywords::PROTECTION_EVERYTHING) {
                add(DamagePreventionKind::ProtectionEverything);
            }
            if keywords.contains(Keywords::PROTECTION_INSTANT_SORCERY)
                && event.source.facts.kinds & (4 | 8) != 0
            {
                add(DamagePreventionKind::ProtectionInstantSorcery);
            }
        }
        out
    }
    fn modify_damage(
        &self,
        choice: ReplacementEffectView,
        event: &mut DamageEvent,
        heals: &mut Vec<ObjectRef>,
    ) {
        match choice.source_card.replacement_effects()[usize::from(choice.ability)] {
            ReplacementSpec::DoubleCreatureDamage
            | ReplacementSpec::DoubleOpponentDamage
            | ReplacementSpec::DoubleEquippedDamage => {
                event.amount = event
                    .amount
                    .checked_mul(2)
                    .capacity("damage replacement quantity exhausted")
            }
            ReplacementSpec::NoncombatDamageToOpponentPlus(amount) => {
                event.amount = event
                    .amount
                    .checked_add(i128::from(amount))
                    .capacity("damage replacement quantity exhausted");
            }
            ReplacementSpec::NoncombatDamagePlusSourcePower => {
                // CR107.1b uses signed intermediate values and clamps the final damage result.
                event.amount = event
                    .amount
                    .checked_add(self.stats(choice.source.card).power)
                    .capacity("damage replacement quantity exhausted")
                    .max(0);
            }
            ReplacementSpec::HealPreviousDamageToSelf => {
                if !heals.contains(&choice.source) {
                    heals.push(choice.source);
                }
            }
            ReplacementSpec::PreventOtherCreatureNoncombatDamage
            | ReplacementSpec::PreventAllDamageToSelf => event.amount = 0,
            _ => unreachable!(),
        }
    }
    pub(super) fn damage_replacement_candidates_exist(&self, events: &[DamageEvent]) -> bool {
        events.iter().any(|event| {
            !self.damage_replacements(event, &[]).is_empty()
                || !self.damage_preventions(event, &[]).is_empty()
        })
    }
    pub(super) fn stage_damage_replacements(&mut self, events: &[DamageEvent]) {
        let resume = if self.state.resolving.is_some() {
            ResumeMode::Program
        } else {
            ResumeMode::Pending
        };
        let state = self.state.replacements.get_or_insert_with(|| {
            Box::new(ReplacementState {
                work: VecDeque::new(),
                active: None,
                attachment: None,
                resume: None,
                mode: resume,
            })
        });
        let mut events = events.to_vec();
        // Simultaneous affected-player choices are made in APNAP order.
        events.sort_by_key(|event| match event.target {
            Target::Player(p) => p != self.state.active,
            Target::Card(id) => self.state.objects[id as usize].controller != self.state.active,
            _ => unreachable!(),
        });
        state.work.push_back(Work::Damage(DamageBatch {
            events,
            heals: Vec::new(),
            used_preventions: Vec::new(),
            ordered_event: false,
            next: 0,
            used: Vec::new(),
        }));
    }
    fn counter_replacements(
        &self,
        p: &CounterPlacement,
        used: &[(ObjectRef, u16)],
    ) -> Vec<ReplacementEffectView> {
        // Counter replacements for a face-up event see the prospective face,
        // including its copy and continuous effects, while the real object
        // remains face down until the event is ready to commit.
        if self
            .state
            .manifest
            .as_ref()
            .and_then(|state| state.face_up_completion)
            == Some(p.object)
            && self.is_face_down(p.object)
        {
            let mut prospective = self.clone();
            prospective.materialize_face_up(p.object);
            return counter_replacements(&prospective, p, used);
        }
        counter_replacements(self, p, used)
    }
    fn commit_counter_placements(
        &mut self,
        mut placements: Vec<CounterPlacement>,
        player_counters: [[u64; 5]; 2],
        proliferated: Option<Player>,
        completion: Option<Box<CounterReflexiveCompletion>>,
        activation_cost: bool,
    ) {
        for placement in &mut placements {
            if placement_prohibited(self, placement) {
                placement.amount = Counters::default();
            }
        }
        if activation_cost {
            self.record_cast_mana_counter_cost(&placements);
        }
        let completed = completion.as_ref().is_some_and(|c| {
            placements.iter().any(|p| {
                p.entry_index.is_none()
                    && c.item
                        .source
                        .is_some_and(|source| p.object == source.object)
                    && p.amount.get(c.kind) > 0
                    && self.on_battlefield(p.object.card)
                    && self.object_ref(p.object.card) == p.object
            })
        });
        let mut lore = Vec::new();
        // As-turned-face-up counters and the status change are one event.
        // Keep the object face down throughout replacement choices, then expose
        // the face immediately before the final counters are committed.
        if let Some(object) = self
            .state
            .manifest
            .as_ref()
            .and_then(|state| state.face_up_completion)
            && placements
                .iter()
                .any(|placement| placement.object == object && placement.entry_index.is_none())
            && self.is_face_down(object)
        {
            self.materialize_face_up(object);
        }
        // Commit the complete world before any placement observer sees it.
        for player in [Player::P0, Player::P1] {
            for kind in crate::cards::PlayerCounterKind::ALL {
                if !self.player_counters_prohibited(player, kind) {
                    self.state.players[player.index()]
                        .add_player_counter(kind, player_counters[player.index()][kind.index()]);
                }
            }
        }
        for p in &placements {
            if let Some(index) = p.entry_index {
                self.state
                    .entry_batch
                    .as_mut()
                    .expect("owned entry counter batch")
                    .moves[index]
                    .counters = p.amount.clone();
            } else if self.on_battlefield(p.object.card)
                && self.object_ref(p.object.card) == p.object
            {
                let before = self.state.objects[p.object.card as usize].counters.lore;
                for kind in CounterKind::ALL {
                    self.add_counter(p.object.card, kind, p.amount.get(kind));
                }
                if p.amount.lore > 0 {
                    lore.push((p.object, before));
                }
            }
        }
        for p in &placements {
            if p.entry_index.is_none() {
                for kind in CounterKind::ALL {
                    self.self_counters_added(p.object, kind, p.amount.get(kind), p.placed_by);
                }
            }
        }
        self.counter_batch_added(
            placements
                .iter()
                .filter(|p| p.entry_index.is_none())
                .map(|p| (p.object, &p.amount, p.placed_by)),
        );
        for (object, before) in lore {
            self.lore_counters_added(object, before);
        }
        if let Some(player) = proliferated {
            self.proliferated_event(player);
        }
        if completed {
            let completion = completion.unwrap();
            self.queue_reflexive(&completion.item, completion.ability, None);
        }
    }
    pub(super) fn stage_counter_placements(
        &mut self,
        placements: Vec<CounterPlacement>,
        mode: ResumeMode,
    ) {
        self.stage_counter_placements_inner(
            placements,
            [[0; 5]; 2],
            None,
            true,
            None,
            mode,
            None,
            Vec::new(),
        );
    }
    #[allow(
        clippy::too_many_arguments,
        reason = "Counter batches retain distinct event and continuation context"
    )]
    fn stage_counter_placements_inner(
        &mut self,
        mut placements: Vec<CounterPlacement>,
        mut player_counters: [[u64; 5]; 2],
        player_placed_by: Option<Player>,
        player_effect: bool,
        proliferated: Option<Player>,
        mode: ResumeMode,
        completion: Option<Box<CounterReflexiveCompletion>>,
        self_effects: Vec<SelfPlayerCounterReplacement>,
    ) {
        for placement in &mut placements {
            if placement_prohibited(self, placement) {
                placement.amount = Counters::default();
            }
        }
        for player in [Player::P0, Player::P1] {
            for kind in crate::cards::PlayerCounterKind::ALL {
                if self.player_counters_prohibited(player, kind)
                    && !self_effects
                        .iter()
                        .any(|effect| effect.player == player && effect.kind == kind)
                {
                    player_counters[player.index()][kind.index()] = 0;
                }
            }
        }
        placements.retain(|p| p.amount.has_any());
        if placements.is_empty() && player_counters == [[0; 5]; 2] && proliferated.is_none() {
            if mode == ResumeMode::ActivationCosts {
                self.record_cast_mana_counter_cost(&[]);
            }
            return;
        }
        placements.sort_by_key(|p| (p.controller != self.state.active, p.object.card));
        if self.state.replacements.is_none()
            && self_effects.is_empty()
            && placements
                .iter()
                .all(|p| self.counter_replacements(p, &[]).is_empty())
            && [Player::P0, Player::P1].into_iter().all(|player| {
                crate::cards::PlayerCounterKind::ALL
                    .into_iter()
                    .all(|kind| {
                        self.player_counter_replacements(
                            player,
                            kind,
                            player_placed_by.unwrap_or(player),
                            player_effect,
                            player_counters[player.index()][kind.index()],
                            &[],
                        )
                        .is_empty()
                    })
            })
        {
            self.commit_counter_placements(
                placements,
                player_counters,
                proliferated,
                completion,
                mode == ResumeMode::ActivationCosts,
            );
            self.finish_face_up_counter_replacement();
            return;
        }
        let active = self.state.active;
        let state = self.state.replacements.get_or_insert_with(|| {
            Box::new(ReplacementState {
                work: VecDeque::new(),
                active: None,
                attachment: None,
                resume: None,
                mode,
            })
        });
        assert_eq!(
            state.mode, mode,
            "one owned replacement continuation at a time"
        );
        let mut player_events = player_counter_events::PlayerCounterEvents::default();
        let origin = self_effects
            .first()
            .map_or(0, |effect| effect.origin_stack_id);
        for player in [active, active.other()] {
            for kind in crate::cards::PlayerCounterKind::ALL {
                let amount = player_counters[player.index()][kind.index()];
                if amount != 0 {
                    player_events.push(player, kind, amount, origin);
                }
            }
        }
        state.work.push_back(Work::Counters(CounterBatch {
            player_events,
            self_effects,
            player_placed_by,
            player_effect: [player_effect; 2],
            next_player: 0,
            proliferated,
            completion,
            placements,
            next: 0,
            used: Vec::new(),
        }));
    }
    pub(super) fn put_source_counter_reflexive(
        &mut self,
        item: &SpellView,
        kind: CounterKind,
        amount: u64,
        ability: u16,
    ) {
        let Some(source) = item.source else {
            return;
        };
        if amount == 0
            || !self.on_battlefield(source.id)
            || self.object_ref(source.id) != source.object
        {
            return;
        }
        let o = &self.state.objects[source.id as usize];
        let placement = CounterPlacement {
            placed_by: item.controller,
            object: source.object,
            card: o.card,
            owner: o.owner,
            controller: o.controller,
            protector: self.battle_protector(source.object),
            subtypes: self.granted_subtypes(source.id),
            kind: self.current_kinds(source.id),
            power: self.stats(source.id).power,
            amount: Counters::of(kind, amount),
            effect: true,
            entry_index: None,
            private: false,
        };
        self.stage_counter_placements_inner(
            vec![placement],
            [[0; 5]; 2],
            None,
            true,
            None,
            ResumeMode::Program,
            Some(Box::new(CounterReflexiveCompletion {
                item: item.clone(),
                kind,
                ability,
            })),
            Vec::new(),
        );
    }
    pub(super) fn activation_cost_counters(
        &mut self,
        object: ObjectRef,
        kind: CounterKind,
        amount: u64,
        player: Player,
    ) {
        if let Some(receipt) = self.take_frozen_mana_counter_cost() {
            if let Some(placements) = receipt {
                // Replacement effects can set `effect` even on a cost event.
                if placements.len() > 1
                    || !self.on_battlefield(object.card)
                    || self.object_ref(object.card) != object
                    || placements.iter().any(|p| {
                        p.object != object || p.entry_index.is_some() || p.placed_by != player
                    })
                    || placements.iter().any(|p| {
                        CounterKind::ALL.into_iter().any(|kind| {
                            self.counter_count(object.card, kind)
                                .checked_add(p.amount.get(kind))
                                .is_none()
                        })
                    })
                {
                    self.fail_frozen_mana_replay();
                } else {
                    // The receipt owns the actual observer packets. Applying
                    // the physical event must not redispatch observers against
                    // a different reconstructed world.
                    for placement in &placements {
                        for kind in CounterKind::ALL {
                            self.add_counter(object.card, kind, placement.amount.get(kind));
                        }
                    }
                    for placement in &placements {
                        for kind in CounterKind::ALL {
                            if placement.amount.get(kind) > 0 {
                                self.record_counter_placement(
                                    object,
                                    player,
                                    kind == CounterKind::PlusOne,
                                );
                            }
                        }
                    }
                }
            }
            return;
        }
        let o = &self.state.objects[object.card as usize];
        let placement = CounterPlacement {
            placed_by: player,
            object,
            card: o.card,
            owner: o.owner,
            controller: o.controller,
            protector: self.battle_protector(object),
            subtypes: self.granted_subtypes(object.card),
            kind: self.current_kinds(object.card),
            power: self.stats(object.card).power,
            amount: Counters::of(kind, amount),
            effect: false,
            entry_index: None,
            private: false,
        };
        self.stage_counter_placements(vec![placement], ResumeMode::ActivationCosts);
    }
    pub(super) fn blight_cost_counters(&mut self, object: ObjectRef, amount: u16, player: Player) {
        let o = &self.state.objects[object.card as usize];
        let placement = CounterPlacement {
            placed_by: player,
            object,
            card: o.card,
            owner: o.owner,
            controller: o.controller,
            protector: self.battle_protector(object),
            subtypes: self.granted_subtypes(object.card),
            kind: self.current_kinds(object.card),
            power: self.stats(object.card).power,
            amount: Counters::of(CounterKind::MinusOne, u64::from(amount)),
            effect: false,
            entry_index: None,
            private: false,
        };
        self.stage_counter_placements(vec![placement], ResumeMode::CastCosts);
        if self.state.replacements.is_none() {
            self.finish_announcement();
        }
    }
    pub(super) fn effect_counters(
        &mut self,
        changes: Vec<(ObjectRef, Counters)>,
        mode: ResumeMode,
    ) {
        self.effect_counters_by(changes, None, [[0; 5]; 2], true, None, mode);
    }
    fn effect_counters_by(
        &mut self,
        changes: Vec<(ObjectRef, Counters)>,
        placed_by: Option<Player>,
        player_counters: [[u64; 5]; 2],
        player_effect: bool,
        proliferated: Option<Player>,
        mode: ResumeMode,
    ) {
        let placements = self.create_counter_placements(changes, placed_by);
        self.stage_counter_placements_inner(
            placements,
            player_counters,
            placed_by.or_else(|| self.state.resolving.as_ref().map(|r| r.item.controller)),
            player_effect,
            proliferated,
            mode,
            None,
            Vec::new(),
        );
    }
    fn create_counter_placements(
        &self,
        changes: Vec<(ObjectRef, Counters)>,
        placed_by: Option<Player>,
    ) -> Vec<CounterPlacement> {
        changes
            .into_iter()
            .filter_map(|(object, amount)| {
                if !self.on_battlefield(object.card) || self.object_ref(object.card) != object {
                    return None;
                }
                let o = &self.state.objects[object.card as usize];
                Some(CounterPlacement {
                    placed_by: placed_by.unwrap_or_else(|| {
                        self.state
                            .resolving
                            .as_ref()
                            .map_or(o.controller, |r| r.item.controller)
                    }),
                    object,
                    card: o.card,
                    owner: o.owner,
                    controller: o.controller,
                    protector: self.battle_protector(object),
                    subtypes: self.granted_subtypes(object.card),
                    kind: self.current_kinds(object.card),
                    power: self.stats(object.card).power,
                    amount,
                    effect: true,
                    entry_index: None,
                    private: false,
                })
            })
            .collect()
    }
    #[cfg(feature = "test-support")]
    pub(super) fn effect_mixed_counters_with_self(
        &mut self,
        changes: Vec<(ObjectRef, Counters)>,
        counters: [[u64; 5]; 2],
        placed_by: Player,
        origin_stack_id: u64,
        effects: Vec<SelfPlayerCounterReplacement>,
        mode: ResumeMode,
    ) {
        assert!(
            self.state
                .resolving
                .as_ref()
                .is_some_and(|r| r.item.id == origin_stack_id)
        );
        assert!(effects.iter().all(|e| e.origin_stack_id == origin_stack_id));
        assert!(
            effects.iter().enumerate().all(|(i, e)| effects[..i]
                .iter()
                .all(|prior| prior.effect != e.effect
                    || prior.player != e.player
                    || prior.kind != e.kind))
        );
        let placements = self.create_counter_placements(changes, Some(placed_by));
        self.stage_counter_placements_inner(
            placements,
            counters,
            Some(placed_by),
            true,
            None,
            mode,
            None,
            effects,
        );
    }
    /// Stage a mixed event from an existing poison-only caller.
    pub(super) fn effect_counters_with_poison(
        &mut self,
        changes: Vec<(ObjectRef, Counters)>,
        player_poison: [u64; 2],
        placed_by: Player,
        mode: ResumeMode,
    ) {
        let mut amounts = [[0; 5]; 2];
        for player in [Player::P0, Player::P1] {
            amounts[player.index()][0] = player_poison[player.index()];
        }
        self.effect_counters_by(changes, Some(placed_by), amounts, true, None, mode);
    }
    pub(super) fn effect_counters_with_players(
        &mut self,
        changes: Vec<(ObjectRef, Counters)>,
        counters: [[u64; 5]; 2],
        placed_by: Player,
        mode: ResumeMode,
    ) {
        self.effect_counters_by(changes, Some(placed_by), counters, true, None, mode);
    }
    #[cfg(feature = "test-support")]
    pub(super) fn effect_player_counters_with_self(
        &mut self,
        player: Player,
        kind: crate::cards::PlayerCounterKind,
        amount: u64,
        origin_stack_id: u64,
        effects: Vec<SelfPlayerCounterReplacement>,
        mode: ResumeMode,
    ) {
        assert!(
            self.state
                .resolving
                .as_ref()
                .is_some_and(|r| r.item.id == origin_stack_id)
        );
        assert!(
            effects.iter().all(|e| e.origin_stack_id == origin_stack_id
                && e.player == player
                && e.kind == kind)
        );
        assert!(
            effects.iter().enumerate().all(|(i, e)| effects[..i]
                .iter()
                .all(|prior| prior.effect != e.effect
                    || prior.player != e.player
                    || prior.kind != e.kind))
        );
        let mut counters = [[0; 5]; 2];
        counters[player.index()][kind.index()] = amount;
        self.stage_counter_placements_inner(
            Vec::new(),
            counters,
            Some(player),
            true,
            None,
            mode,
            None,
            effects,
        );
    }
    pub(super) fn effect_mixed_player_counters_with_self(
        &mut self,
        counters: [[u64; 5]; 2],
        placed_by: Player,
        origin_stack_id: u64,
        effects: Vec<SelfPlayerCounterReplacement>,
        mode: ResumeMode,
    ) {
        assert!(
            self.state
                .resolving
                .as_ref()
                .is_some_and(|r| r.item.id == origin_stack_id)
        );
        assert!(effects.iter().all(|e| e.origin_stack_id == origin_stack_id));
        assert!(
            effects.iter().enumerate().all(|(i, e)| effects[..i]
                .iter()
                .all(|prior| prior.effect != e.effect
                    || prior.player != e.player
                    || prior.kind != e.kind))
        );
        self.stage_counter_placements_inner(
            Vec::new(),
            counters,
            Some(placed_by),
            true,
            None,
            mode,
            None,
            effects,
        );
    }
    // CR 118.11: replacement-modified actions still pay the cost.
    // CR 614.16's effects-only exception does not extend to player counters.
    pub(super) fn pay_poison_counter_cost(&mut self, player: Player, amount: u16) {
        let mut counters = [[0; 5]; 2];
        counters[player.index()][0] = u64::from(amount);
        self.effect_counters_by(
            Vec::new(),
            Some(player),
            counters,
            false,
            None,
            ResumeMode::Program,
        );
    }
    pub(super) fn proliferate_counters(
        &mut self,
        changes: Vec<(ObjectRef, Counters)>,
        counters: [[u64; 5]; 2],
        player: Player,
    ) {
        // Construct permanent placements using the ordinary path, then attach the
        // event before replacement choices start, including for an empty event.
        self.effect_counters_by(
            changes,
            Some(player),
            counters,
            true,
            Some(player),
            ResumeMode::Program,
        );
    }
    pub(super) fn effect_counter(
        &mut self,
        id: CardId,
        kind: CounterKind,
        amount: u64,
        mode: ResumeMode,
    ) {
        if amount == 0 || counters_prohibited(self, self.object_ref(id)) {
            return;
        }
        let placed_by = self
            .state
            .resolving
            .as_ref()
            .map_or(self.state.objects[id as usize].controller, |r| {
                r.item.controller
            });
        self.effect_counter_by(id, kind, amount, placed_by, mode);
    }
    pub(super) fn effect_counter_by(
        &mut self,
        id: CardId,
        kind: CounterKind,
        amount: u64,
        placed_by: Player,
        mode: ResumeMode,
    ) {
        if amount == 0 {
            return;
        }
        if self.state.replacements.is_none()
            && !self.state.static_sources.iter().any(|&id| {
                self.state.objects[id as usize]
                    .card
                    .replacement_effects()
                    .iter()
                    .any(|e| {
                        matches!(
                            e,
                            ReplacementSpec::Counters { .. }
                                | ReplacementSpec::CountersPlacedByController
                        )
                    })
            })
        {
            let before = self.state.objects[id as usize].counters.lore;
            self.add_counter(id, kind, amount);
            let object = self.object_ref(id);
            self.self_counters_added(object, kind, amount, placed_by);
            self.counter_batch_added(std::iter::once((
                object,
                &Counters::of(kind, amount),
                placed_by,
            )));
            if kind == CounterKind::Lore {
                self.lore_counters_added(self.object_ref(id), before);
            }
            return;
        }
        let mut counters = Counters::default();
        counters.add(kind, amount);
        self.effect_counters_by(
            vec![(self.object_ref(id), counters)],
            Some(placed_by),
            [[0; 5]; 2],
            true,
            None,
            mode,
        );
    }
    pub(super) fn defer_empty_explore(&mut self, explorer: ExplorerView, counter_added: bool) {
        self.state
            .replacements
            .as_mut()
            .unwrap()
            .work
            .push_back(Work::EmptyExplore {
                explorer,
                counter_added,
            });
    }
    fn token_replacements(&self, b: &TokenBatch) -> Vec<ReplacementEffectView> {
        let mut out = Vec::new();
        if b.groups.iter().all(|g| g.amount == 0) {
            return out;
        }
        for &id in &self.state.static_sources {
            let o = &self.state.objects[id as usize];
            if o.zone != Zone::Battlefield || o.controller != b.player {
                continue;
            }
            let source = self.object_ref(id);
            for (i, &spec) in o.card.replacement_effects().iter().enumerate() {
                if !self.continuous_fragment_available(id, |fragment| matches!(fragment,
                    crate::cards::StaticAbilityFragment::Replacement(slot, _) if usize::from(*slot) == i)) { continue; }
                let ability = u16::try_from(i).expect("immutable replacement index");
                let applicable = match spec {
                    ReplacementSpec::DoubleTokens { effects_only } => !effects_only || b.effect,
                    ReplacementSpec::AddToken(_) => true,
                    ReplacementSpec::AddArtifactToken(_)
                    | ReplacementSpec::ReplaceArtifactTokens(_) => b
                        .groups
                        .iter()
                        .enumerate()
                        .any(|(index, group)| group.amount > 0 && b.artifact_at(index)),
                    _ => false,
                };
                if applicable && !b.used.contains(&(source, ability)) {
                    out.push(ReplacementEffectView {
                        source,
                        source_card: o.card,
                        ability,
                    });
                }
            }
        }
        out
    }
    fn modify_tokens(choice: ReplacementEffectView, b: &mut TokenBatch) {
        match choice.source_card.replacement_effects()[choice.ability as usize] {
            ReplacementSpec::DoubleTokens { .. } => {
                for g in &mut b.groups {
                    g.amount = g
                        .amount
                        .checked_mul(2)
                        .capacity("replacement token quantity exhausted");
                }
            }
            ReplacementSpec::AddToken(token) | ReplacementSpec::AddArtifactToken(token) => {
                // The added token inherits the event's tapped/attacking instructions.
                let tapped = b.groups[0].tapped;
                if let Some(index) = b.groups.iter().enumerate().position(|(index, g)| {
                    g.token == token
                        && g.tapped == tapped
                        && b.recipes.get(index).is_none_or(Option::is_none)
                }) {
                    let g = &mut b.groups[index];
                    g.amount = g
                        .amount
                        .checked_add(1)
                        .capacity("replacement token quantity exhausted");
                } else {
                    b.groups.push(TokenGroupView {
                        token,
                        amount: 1,
                        tapped,
                    });
                    if !b.recipes.is_empty() {
                        b.recipes.push(None);
                    }
                }
            }
            ReplacementSpec::ReplaceArtifactTokens(token) => {
                for index in 0..b.groups.len() {
                    if b.artifact_at(index) {
                        b.groups[index].token = token;
                        if let Some(recipe) = b.recipes.get_mut(index) {
                            *recipe = None;
                        }
                    }
                }
            }
            _ => unreachable!(),
        }
        b.effect = true;
        b.used.push((choice.source, choice.ability));
    }
    pub(super) fn stage_copy_creation(&mut self, player: Player, recipe: CopyRecipe, amount: u128) {
        self.stage_copy_creation_inner(player, recipe, amount, TokenCompletion::None);
    }
    pub(super) fn stage_temporary_copy_creation(&mut self, item: &SpellView, recipe: CopyRecipe) {
        self.stage_copy_creation_inner(
            item.controller,
            recipe,
            1,
            TokenCompletion::TemporaryCopy(Box::new(item.clone())),
        );
    }
    pub(super) fn stage_copy_creation_inner(
        &mut self,
        player: Player,
        recipe: CopyRecipe,
        amount: u128,
        completion: TokenCompletion,
    ) {
        if amount == 0 {
            return;
        }
        let batch = TokenBatch {
            recipes: vec![Some(recipe)],
            player,
            groups: vec![TokenGroupView {
                token: recipe.definition,
                amount,
                tapped: false,
            }],
            effect: true,
            used: Vec::new(),
            completion,
        };
        self.stage_copy_token_batch(batch);
    }
    /// A single creation event may contain many unrelated frozen copy recipes.
    pub(super) fn stage_copy_recipes(&mut self, player: Player, recipes: Vec<CopyRecipe>) {
        if recipes.is_empty() {
            return;
        }
        let groups = recipes
            .iter()
            .map(|recipe| TokenGroupView {
                token: recipe.definition,
                amount: 1,
                tapped: false,
            })
            .collect();
        self.stage_copy_token_batch(TokenBatch {
            recipes: recipes.into_iter().map(Some).collect(),
            player,
            groups,
            effect: true,
            used: Vec::new(),
            completion: TokenCompletion::None,
        });
    }
    fn stage_copy_token_batch(&mut self, batch: TokenBatch) {
        if matches!(
            batch.completion,
            TokenCompletion::None | TokenCompletion::TemporaryCopy(_)
        ) && self.state.replacements.is_none()
            && self.token_replacements(&batch).is_empty()
        {
            let temporary = match &batch.completion {
                TokenCompletion::TemporaryCopy(item) => Some(item.as_ref()),
                _ => None,
            };
            self.create_copy_groups_inner(batch.player, &batch.groups, &batch.recipes, temporary);
            return;
        }
        let state = self.state.replacements.get_or_insert_with(|| {
            Box::new(ReplacementState {
                work: VecDeque::new(),
                active: None,
                attachment: None,
                resume: None,
                mode: ResumeMode::Program,
            })
        });
        state.work.push_back(Work::Tokens(batch));
    }
    pub(super) fn stage_role_creation(
        &mut self,
        player: Player,
        token: Card,
        recipients: Vec<ObjectRef>,
    ) {
        if recipients.is_empty() {
            return;
        }
        let groups = recipients
            .iter()
            .map(|_| TokenGroupView {
                token,
                amount: 1,
                tapped: false,
            })
            .collect();
        let state = self.state.replacements.get_or_insert_with(|| {
            Box::new(ReplacementState {
                work: VecDeque::new(),
                active: None,
                attachment: None,
                resume: None,
                mode: ResumeMode::Program,
            })
        });
        state.work.push_back(Work::Tokens(TokenBatch {
            recipes: Vec::new(),
            player,
            groups,
            effect: true,
            used: Vec::new(),
            completion: TokenCompletion::RoleAttachMany(recipients),
        }));
    }

    pub(super) fn stage_token_creation(
        &mut self,
        player: Player,
        token: Card,
        amount: u128,
        tapped: bool,
        completion: TokenCompletion,
    ) {
        if amount == 0 {
            return;
        }
        if self.state.replacements.is_none()
            && matches!(completion, TokenCompletion::None)
            && !self.state.static_sources.iter().any(|&id| {
                let o = &self.state.objects[id as usize];
                o.zone == Zone::Battlefield
                    && o.controller == player
                    && o.card.replacement_effects().iter().any(|s| {
                        matches!(
                            s,
                            ReplacementSpec::DoubleTokens { .. }
                                | ReplacementSpec::AddToken(_)
                                | ReplacementSpec::AddArtifactToken(_)
                        ) || matches!(s, ReplacementSpec::ReplaceArtifactTokens(_))
                            && token.is_artifact()
                    })
            })
        {
            self.create_token_batch_capture(player, token, amount, tapped, None, false);
            return;
        }

        let batch = TokenBatch {
            recipes: Vec::new(),
            player,
            groups: vec![TokenGroupView {
                token,
                amount,
                tapped,
            }],
            effect: true,
            used: Vec::new(),
            completion,
        };
        if self.state.replacements.is_none()
            && matches!(batch.completion, TokenCompletion::None)
            && self.token_replacements(&batch).is_empty()
        {
            self.create_token_batch_capture(player, token, amount, tapped, None, false);
            return;
        }
        let state = self.state.replacements.get_or_insert_with(|| {
            Box::new(ReplacementState {
                work: VecDeque::new(),
                active: None,
                attachment: None,
                resume: None,
                mode: ResumeMode::Program,
            })
        });
        state.work.push_back(Work::Tokens(batch));
    }
    pub(super) fn destroy_and_create(
        &mut self,
        targets: Vec<BoundTarget>,
        token: Card,
        amount: u16,
    ) {
        let state = self.state.replacements.get_or_insert_with(|| {
            Box::new(ReplacementState {
                work: VecDeque::new(),
                active: None,
                attachment: None,
                resume: None,
                mode: ResumeMode::Program,
            })
        });
        state.work.push_back(Work::DestroyAndCreate {
            targets,
            token,
            amount,
        });
    }
    pub(super) fn advance_replacements(&mut self, resume_program: bool) -> bool {
        let Some(mut state) = self.state.replacements.take() else {
            return false;
        };
        if state.resume.is_none() {
            state.resume = Some(self.state.pending.clone());
        }
        loop {
            if self.state.mobilize.is_some() || state.attachment.is_some() {
                self.state.replacements = Some(state);
                return true;
            }
            if state.active.is_none() {
                match state.work.pop_front() {
                    Some(Work::AnimateCreated { created, spec }) => {
                        self.animate_created_noncreatures(&created, spec);
                        continue;
                    }
                    Some(Work::CreatedGraveyardCounters {
                        player,
                        created,
                        group,
                    }) => {
                        let cards = self
                            .state
                            .objects
                            .iter()
                            .filter(|object| {
                                object.zone == Zone::Graveyard
                                    && object.owner == player
                                    && !object.is_token()
                                    && !object.is_spell_copy()
                            })
                            .count();
                        let amount = u64::try_from(cards / usize::from(group.get()))
                            .capacity("graveyard counter quantity exhausted");
                        let counters = Counters::of(CounterKind::PlusOne, amount);
                        let placements =
                            self.created_token_counter_placements(player, &created, &counters);
                        if !placements.is_empty() {
                            state.work.push_front(Work::Counters(CounterBatch {
                                player_events: player_counter_events::PlayerCounterEvents::default(
                                ),
                                self_effects: Vec::new(),
                                player_placed_by: None,
                                player_effect: [true; 2],
                                next_player: 0,
                                proliferated: None,
                                completion: None,
                                placements,
                                next: 0,
                                used: Vec::new(),
                            }));
                        }
                        continue;
                    }
                    Some(Work::CreatedCounters {
                        player,
                        created,
                        counters,
                    }) => {
                        let placements =
                            self.created_token_counter_placements(player, &created, &counters);
                        if !placements.is_empty() {
                            state.work.push_front(Work::Counters(CounterBatch {
                                player_events: player_counter_events::PlayerCounterEvents::default(
                                ),
                                self_effects: Vec::new(),
                                player_placed_by: None,
                                player_effect: [true; 2],
                                next_player: 0,
                                proliferated: None,
                                completion: None,
                                placements,
                                next: 0,
                                used: Vec::new(),
                            }));
                        }
                        continue;
                    }
                    Some(Work::AmassSubtype {
                        source,
                        recipient,
                        attach,
                    }) => {
                        self.grant_amass_goblin_type(source, recipient);
                        if attach {
                            self.attach_equipment(source, recipient);
                        }
                        continue;
                    }
                    Some(Work::Proliferate(batch)) => {
                        state.active = Some(Event::Proliferate(batch))
                    }
                    Some(Work::Damage(batch)) => state.active = Some(Event::Damage(batch)),
                    Some(Work::Counters(batch)) => state.active = Some(Event::Counters(batch)),
                    Some(Work::Tokens(batch)) => state.active = Some(Event::Tokens(batch)),
                    Some(Work::EmptyExplore {
                        explorer,
                        counter_added,
                    }) => {
                        self.explore_completed(explorer, None, None, None, counter_added);
                        continue;
                    }
                    Some(Work::DestroyAndCreate {
                        mut targets,
                        token,
                        amount,
                    }) => {
                        if !targets.is_empty() {
                            let bound = targets.remove(0);
                            state.work.push_front(Work::DestroyAndCreate {
                                targets,
                                token,
                                amount,
                            });
                            if self.same_target_object(bound)
                                && let Target::Card(id) = bound.target
                            {
                                let player = self.state.objects[id as usize].controller;
                                if !self.effect_destruction_prevented(id) {
                                    self.move_target_batch(&[bound], Zone::Graveyard);
                                }
                                state.work.push_front(Work::Tokens(TokenBatch {
                                    recipes: Vec::new(),
                                    player,
                                    groups: vec![TokenGroupView {
                                        token,
                                        amount: u128::from(amount),
                                        tapped: false,
                                    }],
                                    effect: true,
                                    used: Vec::new(),
                                    completion: TokenCompletion::None,
                                }));
                                // Leaving may immediately return an exiled permanent.
                                // Finish its entry (including private/replacement choices)
                                // before the next token event of the resolving effect.
                                if let Some(batch) = self.state.entry_batch.as_mut() {
                                    assert!(batch.replacement_resume.is_none());
                                    batch.replacement_resume = Some(state);
                                    return self.advance_entries();
                                }
                            }
                        }
                        continue;
                    }
                    None => {}
                }
            }
            let Some(active) = state.active.as_mut() else {
                self.state.pending = state.resume.take().unwrap();
                self.finish_face_up_counter_replacement();
                if state.mode == ResumeMode::ActivationCosts
                    && self.state.activation_payment.is_some()
                    && self.resume_activation_payment()
                {
                    return self.advance_replacements(resume_program);
                }
                if state.mode == ResumeMode::CastCosts && self.state.announcing.is_some() {
                    self.finish_announcement();
                }
                if resume_program
                    && state.mode == ResumeMode::Program
                    && self.state.resolving.is_some()
                {
                    self.run_program();
                }
                return self.state.replacements.is_some();
            };
            match active {
                Event::Proliferate(batch) => {
                    let choices = self.proliferate_replacements(batch);
                    match choices.as_slice() {
                        [] => {
                            let Some(Event::Proliferate(done)) = state.active.take() else {
                                unreachable!()
                            };
                            self.state
                                .resolving
                                .as_mut()
                                .unwrap()
                                .proliferate
                                .as_mut()
                                .unwrap()
                                .ready = done.amount != 0;
                        }
                        [only] => {
                            if let Some(siblings) = modify_proliferate(
                                only.source_card.replacement_effects()[only.ability as usize],
                                batch,
                                only.source,
                                only.ability,
                            ) {
                                self.state
                                    .resolving
                                    .as_mut()
                                    .unwrap()
                                    .proliferate
                                    .as_mut()
                                    .unwrap()
                                    .pending
                                    .push_front(siblings);
                            }
                        }
                        _ => {
                            self.state.pending = Pending::Replacement {
                                player: batch.player,
                            };
                            self.state.replacements = Some(state);
                            return true;
                        }
                    }
                }
                Event::Damage(batch) => {
                    if batch.next == batch.events.len() {
                        let Some(Event::Damage(batch)) = state.active.take() else {
                            unreachable!()
                        };
                        for reference in batch.heals {
                            if self.on_battlefield(reference.card)
                                && self.object_ref(reference.card) == reference
                            {
                                // Healing removes marked damage, not damage history or deathtouch facts.
                                self.state.objects[reference.card as usize].damage = 0;
                            }
                        }
                        self.commit_damage_batch(&batch.events);
                        continue;
                    }
                    let event = batch.events[batch.next];
                    if !self.damage_event_order(batch).is_empty() {
                        self.state.pending = Pending::Replacement {
                            player: self.damage_recipient_controller(&event),
                        };
                        self.state.replacements = Some(state);
                        return true;
                    }
                    let choices = self.damage_replacements(&event, &batch.used);
                    let preventions = self.damage_preventions(&event, &batch.used_preventions);
                    match choices.len() + preventions.len() {
                        0 => {
                            batch.next += 1;
                            batch.used.clear();
                            batch.used_preventions.clear();
                            batch.ordered_event = false;
                        }
                        1 => {
                            if let Some(only) = choices.first() {
                                self.modify_damage(
                                    *only,
                                    &mut batch.events[batch.next],
                                    &mut batch.heals,
                                );
                                batch.used.push((only.source, only.ability));
                            } else {
                                self.apply_damage_prevention(
                                    preventions[0],
                                    &mut batch.events[batch.next],
                                    &mut batch.used_preventions,
                                );
                            }
                        }
                        _ => {
                            self.state.pending = Pending::Replacement {
                                player: self.damage_recipient_controller(&event),
                            };
                            self.state.replacements = Some(state);
                            return true;
                        }
                    }
                }
                Event::Counters(batch) => {
                    if let Some((player, kind)) = current_player_counter(batch, self.state.active) {
                        let amount = batch.player_events.events()[batch.next_player].amount;
                        let own: Vec<_> = batch
                            .self_effects
                            .iter()
                            .filter(|effect| {
                                amount != 0
                                    && effect.player == player
                                    && effect.kind == kind
                                    && !batch.player_events.events()[batch.next_player]
                                        .applied_self
                                        .contains(&player_counter_events::EffectIdentity {
                                            origin_stack_id: effect.origin_stack_id,
                                            index: effect.effect,
                                        })
                            })
                            .copied()
                            .collect();
                        match own.as_slice() {
                            [only] => {
                                apply_own_player_counter_replacement(
                                    batch,
                                    *only,
                                    self.state.active,
                                );
                                continue;
                            }
                            [] => (),
                            _ => {
                                self.state.pending = Pending::Replacement { player };
                                self.state.replacements = Some(state);
                                return true;
                            }
                        }
                        if self.player_counters_prohibited(player, kind) {
                            batch.player_events.event_mut(batch.next_player).amount = 0;
                            batch.next_player += 1;
                            batch.used.clear();
                            continue;
                        }
                        let choices = self.player_counter_replacements(
                            player,
                            kind,
                            batch.player_placed_by.unwrap_or(player),
                            batch.player_effect[player.index()],
                            amount,
                            &batch.player_events.events()[batch.next_player].applied_external,
                        );
                        match choices.as_slice() {
                            [] => {
                                batch.next_player += 1;
                                batch.used.clear();
                            }
                            [only] => {
                                batch.player_events.event_mut(batch.next_player).amount =
                                    modified_player_counters(
                                        only.source_card.replacement_effects()
                                            [only.ability as usize],
                                        amount,
                                    );
                                let id = batch.player_events.events()[batch.next_player].id;
                                assert!(
                                    batch
                                        .player_events
                                        .mark_external(id, (only.source, only.ability))
                                );
                            }
                            _ => {
                                self.state.pending = Pending::Replacement { player };
                                self.state.replacements = Some(state);
                                return true;
                            }
                        }
                        continue;
                    }
                    if batch.next == batch.placements.len() {
                        let Some(Event::Counters(batch)) = state.active.take() else {
                            unreachable!()
                        };
                        let totals = batch.player_events.totals();
                        self.commit_counter_placements(
                            batch.placements,
                            totals,
                            batch.proliferated,
                            batch.completion,
                            state.mode == ResumeMode::ActivationCosts,
                        );
                        continue;
                    }
                    let p = &mut batch.placements[batch.next];
                    if let Some(index) = p.entry_index
                        && let Some(mut movement) = self
                            .state
                            .entry_batch
                            .as_ref()
                            .and_then(|b| b.moves.get(index))
                            .cloned()
                        && (movement.ghost_vacuum_after.is_some()
                            || entry_power_sensitive(
                                self,
                                movement.controller,
                                movement.before.card,
                            ))
                    {
                        movement.counters = p.amount.clone();
                        if let Some(view) =
                            self.ghost_entry_preview(&movement.replacement_preview())
                        {
                            p.subtypes = view.granted_subtypes;
                            p.kind = view.kinds;
                            p.power = view.power;
                        }
                    }
                    let choices = self.counter_replacements(p, &batch.used);
                    match choices.as_slice() {
                        [] => {
                            batch.next += 1;
                            batch.used.clear();
                        }
                        [only] => {
                            modify_counter(
                                only.source_card.replacement_effects()[only.ability as usize],
                                &mut batch.placements[batch.next],
                            );
                            batch.used.push((only.source, only.ability));
                        }
                        _ => {
                            self.state.pending = Pending::Replacement {
                                player: p.controller,
                            };
                            self.state.replacements = Some(state);
                            return true;
                        }
                    }
                }
                Event::Tokens(batch) => {
                    let choices = self.token_replacements(batch);
                    match choices.as_slice() {
                        [] => {
                            let Some(Event::Tokens(batch)) = state.active.take() else {
                                unreachable!()
                            };
                            if !batch.recipes.is_empty() {
                                let temporary = match &batch.completion {
                                    TokenCompletion::TemporaryCopy(item) => Some(item.as_ref()),
                                    _ => None,
                                };
                                let created = self.create_copy_groups_inner(
                                    batch.player,
                                    &batch.groups,
                                    &batch.recipes,
                                    temporary,
                                );
                                if let TokenCompletion::AnimateNoncreatures(spec) = batch.completion
                                {
                                    state.work.push_front(Work::AnimateCreated {
                                        created: created.clone(),
                                        spec,
                                    });
                                }
                                let counters = match &batch.completion {
                                    TokenCompletion::Counters { kind, amount } => {
                                        Some(Counters::of(*kind, *amount))
                                    }
                                    TokenCompletion::CounterSet(counters) => {
                                        Some(counters.as_ref().clone())
                                    }
                                    _ => None,
                                };
                                if let TokenCompletion::GraveyardGroups(group) = batch.completion {
                                    state.work.push_front(Work::CreatedGraveyardCounters {
                                        player: batch.player,
                                        created: created.clone(),
                                        group,
                                    });
                                }
                                if let Some(counters) = counters {
                                    state.work.push_front(Work::CreatedCounters {
                                        player: batch.player,
                                        created,
                                        counters: Box::new(counters),
                                    });
                                }
                                if let Some(entry) = self.state.entry_batch.as_mut() {
                                    assert!(entry.replacement_resume.is_none());
                                    entry.replacement_resume = Some(state);
                                    return self.advance_entries();
                                }
                                continue;
                            }
                            if matches!(batch.completion, TokenCompletion::Mobilize) {
                                self.start_mobilize_groups(batch.groups);
                            } else {
                                let capture = matches!(
                                    batch.completion,
                                    TokenCompletion::CounterOtherWizards
                                        | TokenCompletion::JobAttach(_)
                                        | TokenCompletion::AttachCreatedTo(_)
                                        | TokenCompletion::Counters { .. }
                                        | TokenCompletion::CounterSet(_)
                                        | TokenCompletion::GraveyardGroups(_)
                                        | TokenCompletion::Keywords { .. }
                                        | TokenCompletion::Reflexive { .. }
                                        | TokenCompletion::ReflexiveCreated { .. }
                                );
                                let created = match &batch.completion {
                                    TokenCompletion::RoleAttach(recipient) => self
                                        .create_attached_role_groups(
                                            batch.player,
                                            &batch.groups,
                                            *recipient,
                                        ),
                                    TokenCompletion::RoleReflexive(c) => self
                                        .create_token_groups_attached(
                                            batch.player,
                                            &batch.groups,
                                            None,
                                            true,
                                            Some(c.recipient),
                                        ),
                                    TokenCompletion::RoleAttachMany(recipients) => self
                                        .create_token_groups_per_recipient(
                                            batch.player,
                                            &batch.groups,
                                            recipients,
                                        ),
                                    _ => self.create_token_groups(
                                        batch.player,
                                        &batch.groups,
                                        None,
                                        capture,
                                    ),
                                };
                                if matches!(batch.completion, TokenCompletion::CounterOtherWizards)
                                {
                                    let others = self
                                        .state
                                        .objects
                                        .iter()
                                        .enumerate()
                                        .filter_map(|(id, object)| {
                                            let id = id as CardId;
                                            let reference = self.object_ref(id);
                                            (object.zone == Zone::Battlefield
                                                && object.controller == batch.player
                                                && object.is_token()
                                                && !created.contains(&reference)
                                                && self.has_subtype(id, "Wizard"))
                                            .then_some(reference)
                                        })
                                        .collect::<Vec<_>>();
                                    let placements = self.created_token_counter_placements(
                                        batch.player,
                                        &others,
                                        &Counters::of(CounterKind::PlusOne, 1),
                                    );
                                    if !placements.is_empty() {
                                        state.work.push_front(Work::Counters(CounterBatch {
                                            player_events:
                                                player_counter_events::PlayerCounterEvents::default(
                                                ),
                                            self_effects: Vec::new(),
                                            player_placed_by: None,
                                            player_effect: [true; 2],
                                            next_player: 0,
                                            proliferated: None,
                                            completion: None,
                                            placements,
                                            next: 0,
                                            used: Vec::new(),
                                        }));
                                    }
                                }
                                let counters = match &batch.completion {
                                    TokenCompletion::Counters { kind, amount } => {
                                        Some(Counters::of(*kind, *amount))
                                    }
                                    TokenCompletion::CounterSet(counters) => {
                                        Some(counters.as_ref().clone())
                                    }
                                    _ => None,
                                };
                                if let TokenCompletion::GraveyardGroups(group) = batch.completion {
                                    state.work.push_front(Work::CreatedGraveyardCounters {
                                        player: batch.player,
                                        created: created.clone(),
                                        group,
                                    });
                                }
                                if let Some(counters) = counters {
                                    let placements = self.created_token_counter_placements(
                                        batch.player,
                                        &created,
                                        &counters,
                                    );
                                    if !placements.is_empty() {
                                        state.work.push_front(Work::Counters(CounterBatch {
                                            player_events:
                                                player_counter_events::PlayerCounterEvents::default(
                                                ),
                                            self_effects: Vec::new(),
                                            player_placed_by: None,
                                            player_effect: [true; 2],
                                            next_player: 0,
                                            proliferated: None,
                                            completion: None,
                                            placements,
                                            next: 0,
                                            used: Vec::new(),
                                        }));
                                    }
                                }
                                if let TokenCompletion::Keywords { source, keywords } =
                                    batch.completion
                                {
                                    let ids = created.iter().map(|r| r.card).collect::<Vec<_>>();
                                    self.grant_keyword_effects(&ids, source, keywords);
                                }
                                if let TokenCompletion::Reflexive { item, ability } =
                                    &batch.completion
                                    && !created.is_empty()
                                {
                                    self.queue_reflexive(item, *ability, None);
                                }
                                if let TokenCompletion::ReflexiveCreated { item, ability } =
                                    &batch.completion
                                    && let Some((&first, rest)) = created.split_first()
                                {
                                    self.queue_reflexive(item, *ability, Some(first));
                                    self.state
                                        .pending_triggers
                                        .last_mut()
                                        .unwrap()
                                        .reflexive_subject
                                        .as_mut()
                                        .unwrap()
                                        .additional_objects = rest.to_vec();
                                }
                                if let TokenCompletion::RoleReflexive(c) = &batch.completion
                                    && created.iter().any(|r| {
                                        self.object_view(r.card).card == c.token
                                            && self.object_view(r.card).attached_to
                                                == Some(c.recipient)
                                    })
                                {
                                    self.queue_reflexive(&c.item, c.ability, Some(c.recipient));
                                }
                                if let TokenCompletion::AttachCreatedTo(recipient) =
                                    batch.completion
                                {
                                    for &equipment in &created {
                                        self.attach_equipment(equipment, recipient);
                                    }
                                }
                                if let TokenCompletion::JobAttach(source) = batch.completion {
                                    let eligible = created
                                        .into_iter()
                                        .filter(|r| self.is_creature(r.card))
                                        .collect::<Vec<_>>();
                                    if self.on_battlefield(source.card)
                                        && self.object_ref(source.card) == source
                                        && !self.is_creature(source.card)
                                    {
                                        match eligible.as_slice() {
                                            [] => {}
                                            [only] => {
                                                self.attach_equipment(source, *only);
                                            }
                                            _ => {
                                                state.attachment = Some((source, eligible));
                                                self.state.pending = Pending::CreatedToken {
                                                    player: batch.player,
                                                };
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        [only] => Self::modify_tokens(*only, batch),
                        _ => {
                            self.state.pending = Pending::Replacement {
                                player: batch.player,
                            };
                            self.state.replacements = Some(state);
                            return true;
                        }
                    }
                }
            }
        }
    }
    pub(super) fn choose_created_token(&mut self, object: ObjectRef) {
        let state = self.state.replacements.as_mut().unwrap();
        let (source, _) = state.attachment.take().unwrap();
        self.attach_equipment(source, object);
        self.advance_replacements(true);
    }
    pub(super) fn visit_created_token_actions(
        &self,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        self.state
            .replacements
            .as_ref()
            .unwrap()
            .attachment
            .as_ref()
            .unwrap()
            .1
            .iter()
            .any(|&object| emit(Action::ChooseCreatedToken { object }))
    }
    pub(super) fn created_token_choice(&self) -> Option<CreatedTokenChoiceView> {
        let (source, candidates) = self.state.replacements.as_ref()?.attachment.as_ref()?;
        Some(CreatedTokenChoiceView {
            source: *source,
            candidates: candidates.clone(),
        })
    }
    pub(super) fn choose_self_counter_replacement(&mut self, effect: u16) {
        let mut state = self.state.replacements.take().unwrap();
        let Event::Counters(batch) = state.active.as_mut().unwrap() else {
            unreachable!()
        };
        let (player, kind) = current_player_counter(batch, self.state.active).unwrap();
        let selected = batch
            .self_effects
            .iter()
            .find(|e| {
                e.effect == effect
                    && e.player == player
                    && e.kind == kind
                    && !batch.player_events.events()[batch.next_player]
                        .applied_self
                        .contains(&player_counter_events::EffectIdentity {
                            origin_stack_id: e.origin_stack_id,
                            index: effect,
                        })
            })
            .copied()
            .unwrap();
        apply_own_player_counter_replacement(batch, selected, self.state.active);
        self.state.replacements = Some(state);
        self.advance_replacements(true);
    }
    pub(super) fn choose_replacement(&mut self, source: ObjectRef, ability: u16) {
        if self.state.replacements.is_none() {
            self.apply_connive_replacement(source, ability);
            self.run_program();
            return;
        }
        let mut state = self.state.replacements.take().unwrap();
        match state.active.as_mut().unwrap() {
            Event::Proliferate(batch) => {
                let choice = self
                    .proliferate_replacements(batch)
                    .into_iter()
                    .find(|c| c.source == source && c.ability == ability)
                    .unwrap();
                if let Some(siblings) = modify_proliferate(
                    choice.source_card.replacement_effects()[ability as usize],
                    batch,
                    source,
                    ability,
                ) {
                    self.state
                        .resolving
                        .as_mut()
                        .unwrap()
                        .proliferate
                        .as_mut()
                        .unwrap()
                        .pending
                        .push_front(siblings);
                }
            }
            Event::Damage(batch) => {
                let choice = self
                    .damage_replacements(&batch.events[batch.next], &batch.used)
                    .into_iter()
                    .find(|c| c.source == source && c.ability == ability)
                    .unwrap();
                self.modify_damage(choice, &mut batch.events[batch.next], &mut batch.heals);
                batch.used.push((source, ability));
            }
            Event::Counters(batch)
                if current_player_counter(batch, self.state.active).is_some() =>
            {
                let (player, kind) = current_player_counter(batch, self.state.active).unwrap();
                let amount = batch.player_events.events()[batch.next_player].amount;
                let choice = self
                    .player_counter_replacements(
                        player,
                        kind,
                        batch.player_placed_by.unwrap_or(player),
                        batch.player_effect[player.index()],
                        amount,
                        &batch.player_events.events()[batch.next_player].applied_external,
                    )
                    .into_iter()
                    .find(|c| c.source == source && c.ability == ability)
                    .unwrap();
                batch.player_events.event_mut(batch.next_player).amount = modified_player_counters(
                    choice.source_card.replacement_effects()[ability as usize],
                    amount,
                );
                let id = batch.player_events.events()[batch.next_player].id;
                assert!(batch.player_events.mark_external(id, (source, ability)));
            }
            Event::Counters(batch) => {
                let p = &mut batch.placements[batch.next];
                let choice = self
                    .counter_replacements(p, &batch.used)
                    .into_iter()
                    .find(|c| c.source == source && c.ability == ability)
                    .unwrap();
                modify_counter(
                    choice.source_card.replacement_effects()[ability as usize],
                    p,
                );
                batch.used.push((source, ability));
            }
            Event::Tokens(batch) => {
                let choice = self
                    .token_replacements(batch)
                    .into_iter()
                    .find(|c| c.source == source && c.ability == ability)
                    .unwrap();
                Self::modify_tokens(choice, batch);
            }
        }
        self.state.replacements = Some(state);
        self.advance_replacements(true);
    }
    pub(super) fn choose_damage_prevention(
        &mut self,
        source: ObjectRef,
        kind: DamagePreventionKind,
    ) {
        let mut state = self.state.replacements.take().unwrap();
        let Event::Damage(batch) = state.active.as_mut().unwrap() else {
            unreachable!()
        };
        assert!(
            self.damage_preventions(&batch.events[batch.next], &batch.used_preventions)
                .iter()
                .any(|choice| choice.source == source && choice.kind == kind)
        );
        let choice = self
            .damage_preventions(&batch.events[batch.next], &batch.used_preventions)
            .into_iter()
            .find(|choice| choice.source == source && choice.kind == kind)
            .unwrap();
        self.apply_damage_prevention(
            choice,
            &mut batch.events[batch.next],
            &mut batch.used_preventions,
        );
        self.state.replacements = Some(state);
        self.advance_replacements(true);
    }
    fn apply_damage_prevention(
        &mut self,
        choice: DamagePreventionView,
        event: &mut DamageEvent,
        used: &mut Vec<(ObjectRef, DamagePreventionKind)>,
    ) {
        if choice.kind == DamagePreventionKind::ShieldCounter {
            self.remove_shield(choice.source.card);
        }
        if event.preventable && !self.damage_prevention_prohibited() {
            event.amount = 0;
        }
        used.push((choice.source, choice.kind));
    }
    fn damage_event_order(&self, batch: &DamageBatch) -> Vec<DamageEventOrderView> {
        if batch.ordered_event {
            return Vec::new();
        }
        let event = batch.events[batch.next];
        if event.amount <= 0 {
            return Vec::new();
        }
        let Target::Card(id) = event.target else {
            return Vec::new();
        };
        if self.counter_count(id, CounterKind::Shield) == 0 {
            return Vec::new();
        }
        let choices: Vec<_> = batch
            .events
            .iter()
            .enumerate()
            .skip(batch.next)
            .filter(|(_, candidate)| candidate.target == event.target && candidate.amount > 0)
            .map(|(index, &event)| DamageEventOrderView {
                index: u32::try_from(index).expect("owned damage event index"),
                event,
            })
            .collect();
        if choices.len() > 1 {
            choices
        } else {
            Vec::new()
        }
    }
    pub(super) fn choose_damage_event(&mut self, index: u32) {
        let mut state = self.state.replacements.take().unwrap();
        let Event::Damage(batch) = state.active.as_mut().unwrap() else {
            unreachable!()
        };
        assert!(
            self.damage_event_order(batch)
                .iter()
                .any(|choice| choice.index == index)
        );
        batch.events.swap(batch.next, index as usize);
        batch.ordered_event = true;
        self.state.replacements = Some(state);
        self.advance_replacements(true);
    }
    pub(super) fn visit_replacement_actions(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        if self.state.replacements.is_none() {
            return self.connive_replacements().into_iter().any(|c| {
                emit(Action::ChooseReplacement {
                    source: c.source,
                    ability: c.ability,
                })
            });
        }
        let active = self
            .state
            .replacements
            .as_ref()
            .unwrap()
            .active
            .as_ref()
            .unwrap();
        if let Event::Damage(batch) = active {
            let choices = self.damage_event_order(batch);
            if !choices.is_empty() {
                return choices.into_iter().any(|choice| {
                    emit(Action::ChooseDamageEvent {
                        index: choice.index,
                    })
                });
            }
        }
        if let Event::Counters(batch) = active
            && let Some((player, kind)) = current_player_counter(batch, self.state.active)
        {
            let amount = batch.player_events.events()[batch.next_player].amount;
            let own: Vec<_> = batch
                .self_effects
                .iter()
                .filter(|effect| {
                    amount != 0
                        && effect.player == player
                        && effect.kind == kind
                        && !batch.player_events.events()[batch.next_player]
                            .applied_self
                            .contains(&player_counter_events::EffectIdentity {
                                origin_stack_id: effect.origin_stack_id,
                                index: effect.effect,
                            })
                })
                .collect();
            if !own.is_empty() {
                return own.into_iter().any(|effect| {
                    emit(Action::ChooseSelfCounterReplacement {
                        effect: effect.effect,
                    })
                });
            }
        }
        let choices = match active {
            Event::Proliferate(b) => self.proliferate_replacements(b),
            Event::Damage(b) => self.damage_replacements(&b.events[b.next], &b.used),
            Event::Counters(b) if current_player_counter(b, self.state.active).is_some() => {
                let (player, kind) = current_player_counter(b, self.state.active).unwrap();
                self.player_counter_replacements(
                    player,
                    kind,
                    b.player_placed_by.unwrap_or(player),
                    b.player_effect[player.index()],
                    b.player_events.events()[b.next_player].amount,
                    &b.player_events.events()[b.next_player].applied_external,
                )
            }
            Event::Counters(b) => self.counter_replacements(&b.placements[b.next], &b.used),
            Event::Tokens(b) => self.token_replacements(b),
        };
        if choices.into_iter().any(|c| {
            emit(Action::ChooseReplacement {
                source: c.source,
                ability: c.ability,
            })
        }) {
            return true;
        }
        if let Event::Damage(batch) = active {
            return self
                .damage_preventions(&batch.events[batch.next], &batch.used_preventions)
                .into_iter()
                .any(|choice| {
                    emit(Action::ChooseDamagePrevention {
                        source: choice.source,
                        prevention: choice.kind,
                    })
                });
        }
        false
    }
    pub(super) fn replacement_choice(&self, viewer: Player) -> Option<ReplacementChoiceView> {
        if !matches!(self.state.pending, Pending::Replacement { .. }) {
            return None;
        }
        if self.state.replacements.is_none() {
            return self.connive_replacement_choice();
        }
        match self.state.replacements.as_ref()?.active.as_ref()? {
            Event::Proliferate(batch) => Some(ReplacementChoiceView {
                self_counter_effects: Vec::new(),
                kind: "Proliferate",
                object: None,
                counters: None,
                player_counters: None,
                proliferate: Some(ProliferateReplacementView {
                    player: batch.player,
                    amount: batch.amount,
                }),
                tokens: Vec::new(),
                copy_recipes: Vec::new(),
                role_attachments: Vec::new(),
                effects: self.proliferate_replacements(batch),
                damage: None,
                damage_preventions: Vec::new(),
                heals_before_damage: Vec::new(),
                damage_event_order: Vec::new(),
            }),
            Event::Damage(batch) => Some(ReplacementChoiceView {
                self_counter_effects: Vec::new(),
                kind: if self.damage_event_order(batch).is_empty() {
                    "Damage"
                } else {
                    "DamageEvent"
                },
                object: None,
                counters: None,
                player_counters: None,
                proliferate: None,
                tokens: Vec::new(),
                copy_recipes: Vec::new(),
                role_attachments: Vec::new(),
                effects: if self.damage_event_order(batch).is_empty() {
                    self.damage_replacements(&batch.events[batch.next], &batch.used)
                } else {
                    Vec::new()
                },
                damage: self
                    .damage_event_order(batch)
                    .is_empty()
                    .then_some(batch.events[batch.next]),
                damage_preventions: if self.damage_event_order(batch).is_empty() {
                    self.damage_preventions(&batch.events[batch.next], &batch.used_preventions)
                } else {
                    Vec::new()
                },
                heals_before_damage: batch.heals.clone(),
                damage_event_order: self.damage_event_order(batch),
            }),
            Event::Counters(batch)
                if current_player_counter(batch, self.state.active).is_some() =>
            {
                let (player, kind) = current_player_counter(batch, self.state.active).unwrap();
                Some(ReplacementChoiceView {
                    self_counter_effects: batch
                        .self_effects
                        .iter()
                        .filter(|e| {
                            e.player == player
                                && e.kind == kind
                                && !batch.player_events.events()[batch.next_player]
                                    .applied_self
                                    .contains(&player_counter_events::EffectIdentity {
                                        origin_stack_id: e.origin_stack_id,
                                        index: e.effect,
                                    })
                        })
                        .copied()
                        .collect(),
                    kind: "PlayerCounters",
                    object: None,
                    counters: None,
                    player_counters: Some(PlayerCounterPlacementView {
                        player,
                        kind,
                        amount: batch.player_events.events()[batch.next_player].amount,
                        placed_by: batch
                            .player_placed_by
                            .expect("player counter effect controller"),
                        effect: batch.player_effect[player.index()],
                    }),
                    proliferate: None,
                    tokens: Vec::new(),
                    copy_recipes: Vec::new(),
                    role_attachments: Vec::new(),
                    effects: if batch.self_effects.iter().any(|e| {
                        e.player == player
                            && e.kind == kind
                            && !batch.player_events.events()[batch.next_player]
                                .applied_self
                                .contains(&player_counter_events::EffectIdentity {
                                    origin_stack_id: e.origin_stack_id,
                                    index: e.effect,
                                })
                    }) {
                        Vec::new()
                    } else {
                        self.player_counter_replacements(
                            player,
                            kind,
                            batch.player_placed_by.unwrap_or(player),
                            batch.player_effect[player.index()],
                            batch.player_events.events()[batch.next_player].amount,
                            &batch.player_events.events()[batch.next_player].applied_external,
                        )
                    },
                    damage: None,
                    damage_preventions: Vec::new(),
                    heals_before_damage: Vec::new(),
                    damage_event_order: Vec::new(),
                })
            }
            Event::Counters(batch) => {
                let p = &batch.placements[batch.next];
                if p.private && !self.shares_information(viewer, p.owner) {
                    return None;
                }
                Some(ReplacementChoiceView {
                    self_counter_effects: Vec::new(),
                    damage: None,
                    damage_preventions: Vec::new(),
                    heals_before_damage: Vec::new(),
                    damage_event_order: Vec::new(),
                    kind: "Counters",
                    object: Some(p.object),
                    counters: Some(p.amount.clone()),
                    player_counters: None,
                    proliferate: None,
                    tokens: Vec::new(),
                    copy_recipes: Vec::new(),
                    role_attachments: Vec::new(),
                    effects: self.counter_replacements(p, &batch.used),
                })
            }
            Event::Tokens(batch) => Some(ReplacementChoiceView {
                self_counter_effects: Vec::new(),
                damage: None,
                damage_preventions: Vec::new(),
                heals_before_damage: Vec::new(),
                damage_event_order: Vec::new(),
                kind: "Tokens",
                object: None,
                counters: None,
                player_counters: None,
                proliferate: None,
                tokens: batch.groups.clone(),
                copy_recipes: batch.recipes.clone(),
                role_attachments: match &batch.completion {
                    TokenCompletion::RoleAttach(recipient) => batch
                        .groups
                        .iter()
                        .map(|g| g.token.role().map(|_| *recipient))
                        .collect(),
                    TokenCompletion::RoleReflexive(completion) => batch
                        .groups
                        .iter()
                        .map(|group| group.token.role().map(|_| completion.recipient))
                        .collect(),
                    TokenCompletion::RoleAttachMany(recipients) => batch
                        .groups
                        .iter()
                        .enumerate()
                        .map(|(index, g)| {
                            g.token.role().and_then(|_| recipients.get(index).copied())
                        })
                        .collect(),
                    _ => Vec::new(),
                },
                effects: self.token_replacements(batch),
            }),
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Serialize)]
pub struct CreatedTokenChoiceView {
    pub source: ObjectRef,
    pub candidates: Vec<ObjectRef>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    fn jace_recipients(&self, player: Player) -> Vec<ObjectRef> {
        self.state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, o)| {
                o.zone == Zone::Battlefield
                    && o.controller == player
                    && o.is_token()
                    && self.current_kinds(*id as CardId) & 64 != 0
                    && self.has_subtype(*id as CardId, "Jace")
            })
            .map(|(id, _)| self.object_ref(id as CardId))
            .collect()
    }
    pub(super) fn ensure_jace_token(&mut self, player: Player) {
        if self.jace_recipients(player).is_empty() {
            self.create_tokens(player, Card::JacePlaneswalkerToken, 1, false);
        }
    }
    pub(super) fn finish_empower_jace(
        &mut self,
        player: Player,
        recipient: ObjectRef,
        amount: u64,
    ) {
        self.effect_counters_by(
            vec![(recipient, Counters::of(CounterKind::Loyalty, amount))],
            Some(player),
            [[0; 5]; 2],
            true,
            None,
            ResumeMode::Program,
        );
    }
    pub(super) fn empower_jace(&mut self, item: &SpellView, amount: u64) -> bool {
        let player = item.controller;
        let recipients = self.jace_recipients(player);
        if recipients.len() == 1 {
            self.finish_empower_jace(player, recipients[0], amount);
        } else if recipients.len() > 1 {
            let mut filter = evidence::cost(0).filter;
            filter.subtype = Some("Jace");
            filter.kinds = 64;
            filter.tokens = Some(true);
            return self.start_effect_choice(
                item,
                EffectChoiceKind::EmpowerJace(amount),
                PlayerSet::Controller,
                filter,
                1,
                None,
            );
        }
        false
    }
    fn army_recipients(&self, player: Player) -> Vec<ObjectRef> {
        self.state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, o)| {
                o.zone == Zone::Battlefield
                    && o.controller == player
                    && self.is_creature(*id as CardId)
                    && self.has_subtype(*id as CardId, "Army")
            })
            .map(|(id, _)| self.object_ref(id as CardId))
            .collect()
    }
    pub(super) fn ensure_goblin_army(&mut self, player: Player) {
        // Preserve the allocation-free existence check of the fixed-amount path.
        if !self.state.objects.iter().enumerate().any(|(id, object)| {
            object.zone == Zone::Battlefield
                && object.controller == player
                && self.is_creature(id as CardId)
                && self.has_subtype(id as CardId, "Army")
        }) {
            self.create_tokens(player, Card::GoblinArmyToken, 1, false);
        }
    }
    pub(super) fn amass_goblins_for(
        &mut self,
        item: &SpellView,
        player: Player,
        amount: u64,
    ) -> bool {
        let armies = self.army_recipients(player);
        if armies.len() == 1 {
            self.finish_amass_goblins(item, player, armies[0], amount);
        } else if armies.len() > 1 {
            let mut filter = evidence::cost(0).filter;
            filter.subtype = Some("Army");
            filter.kinds = 1;
            let mut chooser = item.clone();
            chooser.controller = player;
            return self.start_effect_choice(
                &chooser,
                EffectChoiceKind::AmassGoblins(amount),
                PlayerSet::Controller,
                filter,
                1,
                None,
            );
        }
        false
    }
    pub(super) fn finish_amass_goblins(
        &mut self,
        item: &SpellView,
        player: Player,
        recipient: ObjectRef,
        amount: u64,
    ) {
        let Some(source) = item.source else {
            return;
        };
        // The paused program owns this immutable continuation. Army selection
        // reuses the ordinary public amass choice without a global last result.
        let attach = self.state.resolving.as_ref().is_some_and(|resolution| {
            matches!(
                resolution
                    .pc
                    .checked_sub(1)
                    .and_then(|pc| resolution.program.get(pc)),
                Some(Instruction::AmassGoblinsThenAttach(_))
            )
        });
        self.effect_counters_by(
            vec![(recipient, Counters::of(CounterKind::PlusOne, amount))],
            Some(player),
            [[0; 5]; 2],
            true,
            None,
            ResumeMode::Program,
        );
        if let Some(state) = self.state.replacements.as_mut() {
            state.work.push_back(Work::AmassSubtype {
                source: source.object,
                recipient,
                attach,
            });
        } else {
            self.grant_amass_goblin_type(source.object, recipient);
            if attach {
                self.attach_equipment(source.object, recipient);
            }
        }
    }
}
