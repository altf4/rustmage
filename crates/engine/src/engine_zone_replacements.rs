//! Typed graveyard-event replacements and rare owned competing-effect choices.
use super::ability_world::CharacteristicWorld;
use super::*;
use crate::cards::ZoneReplacementRule;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn affected_player(before: &ObjectView) -> Player {
    match before.zone {
        Zone::Battlefield | Zone::Stack => before.controller,
        _ => before.owner,
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneReplacementKind {
    RevealShuffle,
    DreamExile,
    AdventureExile,
    OmenShuffle,
    /// The event is prevented; the object remains in its original zone.
    Prevent,
    Exile,
    /// Distinct replacement identity; only the selected source creates a reflexive packet.
    ExileReflexive {
        source: ObjectRef,
        ability: u16,
    },
    /// Distinct from other exile replacements: this source receives the link and life gain.
    ExileLinked {
        source: ObjectRef,
    },
    Battlefield,
    Shuffle,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ZoneRevealView {
    pub card: Card,
    pub owner: Player,
    pub from: Zone,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ZoneReplacementPickView {
    pub object: ObjectRef,
    pub player: Player,
    pub effect: ZoneReplacementKind,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ZoneReplacementChoiceView {
    pub card: Card,
    pub owner: Player,
    pub from: Zone,
    pub choices: Vec<ZoneReplacementKind>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub prior_public_choices: Vec<ZoneReplacementPickView>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ZoneRequest {
    object: ObjectRef,
    requested: Zone,
    affected: Player,
    public: bool,
    view: ZoneReplacementChoiceView,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ZonePick {
    object: ObjectRef,
    requested: Zone,
    effect: ZoneReplacementKind,
    player: Player,
    public: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Request {
    Zone(ZoneRequest),
    LifeGain(LifeGainReplacementChoice),
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pick {
    Zone(ZonePick),
    LifeGain {
        player: Player,
        amount: i128,
        source: ObjectRef,
        ability: u16,
    },
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy)]
enum Selection {
    Zone(ZoneReplacementKind),
    LifeGain { source: ObjectRef, ability: u16 },
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct Base {
    state: Box<State>,
    bytes: usize,
    actor: Player,
    action: Action,
    picks: SharedVec<Pick>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct Choice {
    base: Base,
    request: Request,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct Replay {
    base: Base,
    next: usize,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ZoneState {
    pub dream_exiles: Vec<DreamExileView>,
    choice: Option<Box<Choice>>,
    replay: Option<Box<Replay>>,
    reveals: SharedVec<ZoneRevealView>,
    graveyard_exile_spells: SharedVec<ObjectRef>,
}
/// A checkpoint fixture for conservative schema projection tests.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[cfg(feature = "test-support")]
pub(super) fn test_checkpoint(state: State, object: ObjectRef, owner: Player) -> ZoneState {
    ZoneState {
        choice: Some(Box::new(Choice {
            base: Base {
                state: Box::new(state),
                bytes: 0,
                actor: owner,
                action: Action::Pass,
                picks: SharedVec::default(),
            },
            request: Request::Zone(ZoneRequest {
                object,
                requested: Zone::Graveyard,
                affected: owner,
                public: false,
                view: ZoneReplacementChoiceView {
                    card: Card::Forest,
                    owner,
                    from: Zone::Hand,
                    choices: vec![ZoneReplacementKind::Prevent],
                    prior_public_choices: Vec::new(),
                },
            }),
        })),
        ..ZoneState::default()
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default)]
pub(super) struct FrozenRules {
    pub dream: Option<DreamExileView>,
    pub prevent: bool,
    pub shuffle_self: bool,
    pub shuffle_spell: bool,
    pub exile_spell: bool,
    pub discard_to_battlefield: bool,
    pub reflexive_exiles: Vec<(u16, ObjectView)>,
    pub linked_death_exiles: Vec<LinkedDeathExile>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct LinkedDeathExile {
    pub source: ObjectRef,
    pub ability_origin: Option<u64>,
    pub controller: Player,
    pub life: u8,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ZoneShuffle {
    Plain,
    Reveal,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ZoneMoveCompletion {
    Dream,
    Adventure(Player),
    LinkedDeathExile(LinkedDeathExile),
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Plan {
    pub completion: Option<ZoneMoveCompletion>,
    pub destination: Zone,
    pub prevented: bool,
    pub shuffle: Option<ZoneShuffle>,
    pub reflexive: Option<(u16, Box<ObjectView>)>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ZoneState {
    // Choices and replays own checkpoints; passive reveals and object links do not.
    pub(super) fn has_checkpoint(&self) -> bool {
        self.choice.is_some() || self.replay.is_some()
    }
    #[cfg(feature = "test-support")]
    pub(super) fn schema191_passive_safe(&self, new_card: impl Fn(Card) -> bool) -> bool {
        !self.has_checkpoint() && self.reveals.iter().all(|view| !new_card(view.card))
    }
    pub fn bytes(&self) -> usize {
        let base_bytes = |base: &Base| {
            base.bytes
                + base.picks.len() * std::mem::size_of::<Pick>()
                + match &base.action {
                    Action::Activate {
                        counter_payment, ..
                    }
                    | Action::ActivateMana {
                        counter_payment, ..
                    }
                    | Action::ActivateBorrowedMana {
                        counter_payment, ..
                    }
                    | Action::ActivateGraveyardBorrowedMana {
                        counter_payment, ..
                    }
                    | Action::AnnounceBorrowedMana {
                        counter_payment, ..
                    }
                    | Action::AnnounceGraveyardBorrowedMana {
                        counter_payment, ..
                    }
                    | Action::AnnounceBorrowedAbility {
                        counter_payment, ..
                    }
                    | Action::AnnounceGraveyardBorrowedAbility {
                        counter_payment, ..
                    }
                    | Action::BeginActivate {
                        counter_payment, ..
                    }
                    | Action::AnnounceActivate {
                        counter_payment, ..
                    }
                    | Action::AnnounceMana {
                        counter_payment, ..
                    } => counter_payment.owned_bytes(),
                    _ => 0,
                }
        };
        std::mem::size_of::<Self>()
            + self.reveals.len() * std::mem::size_of::<ZoneRevealView>()
            + self.graveyard_exile_spells.len() * std::mem::size_of::<ObjectRef>()
            + self.dream_exiles.len() * std::mem::size_of::<DreamExileView>()
            + self.choice.as_ref().map_or(0, |c| {
                std::mem::size_of::<Choice>()
                    + base_bytes(&c.base)
                    + match &c.request {
                        Request::Zone(request) => {
                            request.view.choices.len() * std::mem::size_of::<ZoneReplacementKind>()
                        }
                        Request::LifeGain(request) => {
                            request.effects.len() * std::mem::size_of::<LifeGainReplacementEffect>()
                        }
                    }
            })
            + self
                .replay
                .as_ref()
                .map_or(0, |r| std::mem::size_of::<Replay>() + base_bytes(&r.base))
    }
}
#[cfg(feature = "test-support")]
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
thread_local! {
    pub(super) static TEST_INVALIDATE_STAGED_COST: std::cell::Cell<Option<ObjectRef>> = const { std::cell::Cell::new(None) };
    pub(super) static TEST_FAKE_ENTRY_LIFE: std::cell::Cell<Option<ObjectRef>> = const { std::cell::Cell::new(None) };
    pub(super) static TEST_REDIRECT_EXILE_TO_BATTLEFIELD: std::cell::Cell<Option<ObjectRef>> = const { std::cell::Cell::new(None) };
    pub(super) static TEST_PREVENT_ZONE_MOVE: std::cell::Cell<Option<ObjectRef>> = const { std::cell::Cell::new(None) };
    pub(super) static TEST_PREVENT_PROVIDER: std::cell::Cell<Option<ObjectRef>> = const { std::cell::Cell::new(None) };
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn card_provides_prevention(card: Card) -> bool {
    card.static_ability_groups().iter().any(|group| {
        group.fragments.iter().any(|fragment| {
            matches!(
                fragment,
                crate::cards::StaticAbilityFragment::PreventControlledPermanentGraveyardMove
            )
        })
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn register_stack_zone_replacements(&mut self, item: &SpellView) {
        if item.kind != StackKind::Spell || item.is_copy {
            return;
        }
        let (Some(cast), Some(source)) = (item.cast, item.source) else {
            return;
        };
        if cast.exile_on_graveyard {
            let rows = &mut self
                .state
                .zone_rules
                .get_or_insert_default()
                .graveyard_exile_spells;
            if !rows.contains(&source.object) {
                rows.push(source.object);
            }
        } else if cast.face == CastFace::Omen && self.state.objects[source.id as usize].flashback {
            // Competing stack-exit and Omen replacements need action replay.
            self.state.zone_rules.get_or_insert_default();
        }
    }
    pub(super) fn forget_stack_graveyard_exile(&mut self, object: ObjectRef) {
        self.forget_dream_exile(object);
        if let Some(rules) = &mut self.state.zone_rules
            && rules.graveyard_exile_spells.contains(&object)
        {
            rules
                .graveyard_exile_spells
                .retain(|reference| *reference != object);
        }
    }
    fn stack_graveyard_exile(&self, object: ObjectRef) -> bool {
        self.state
            .zone_rules
            .as_ref()
            .is_some_and(|rules| rules.graveyard_exile_spells.contains(&object))
    }

    pub(super) fn register_zone_rule_card(&mut self, card: Card) {
        if card.zone_replacement().is_some()
            || card_provides_prevention(card)
            || life_gain::can_double(card)
        {
            self.state.zone_rules.get_or_insert_default();
        }
    }
    pub(super) fn record_zone_reveal(&mut self, card: Card, owner: Player, from: Zone) {
        self.state
            .zone_rules
            .get_or_insert_default()
            .reveals
            .push(ZoneRevealView { card, owner, from });
    }
    pub(super) fn clear_zone_reveals(&mut self) {
        if let Some(rules) = &mut self.state.zone_rules {
            rules.reveals.clear();
        }
    }
    pub(super) fn zone_reveals(&self) -> Vec<ZoneRevealView> {
        self.state
            .zone_rules
            .as_ref()
            .map_or_else(Vec::new, |r| r.reveals.to_vec())
    }
    pub(super) fn zone_choice_view(&self, viewer: Player) -> Option<ZoneReplacementChoiceView> {
        if !matches!(self.state.pending, Pending::ZoneReplacement { .. }) {
            return None;
        }
        let choice = self.state.zone_rules.as_ref()?.choice.as_ref()?;
        let Request::Zone(request) = &choice.request else {
            return None;
        };
        self.shares_information(viewer, request.affected).then(|| {
            let mut view = request.view.clone();
            view.prior_public_choices = choice
                .base
                .picks
                .iter()
                .filter_map(|pick| {
                    let Pick::Zone(pick) = pick else {
                        return None;
                    };
                    pick.public.then_some(ZoneReplacementPickView {
                        object: pick.object,
                        player: pick.player,
                        effect: pick.effect,
                    })
                })
                .collect();
            view
        })
    }
    pub(super) fn life_gain_choice_view(
        &self,
        viewer: Player,
    ) -> Option<LifeGainReplacementChoice> {
        if !matches!(self.state.pending, Pending::LifeGainReplacement { .. }) {
            return None;
        }
        let choice = self.state.zone_rules.as_ref()?.choice.as_ref()?;
        let Request::LifeGain(request) = &choice.request else {
            return None;
        };
        self.shares_information(viewer, request.player)
            .then(|| request.clone())
    }
    pub(super) fn ordered_event_choice_actions(
        &self,
        mut emit: impl FnMut(Action) -> bool,
    ) -> bool {
        let choice = self
            .state
            .zone_rules
            .as_ref()
            .unwrap()
            .choice
            .as_ref()
            .unwrap();
        match &choice.request {
            Request::Zone(request) => request
                .view
                .choices
                .iter()
                .any(|&effect| emit(Action::ChooseZoneReplacement { effect })),
            Request::LifeGain(request) => request.effects.iter().any(|effect| {
                emit(Action::ChooseReplacement {
                    source: effect.source,
                    ability: effect.ability,
                })
            }),
        }
    }
    fn next_ordered_pick(&mut self) -> Option<Pick> {
        let replay = self.state.zone_rules.as_mut()?.replay.as_mut()?;
        let pick = *replay.base.picks.get(replay.next)?;
        replay.next += 1;
        Some(pick)
    }
    pub(super) fn select_life_gain_replacement(
        &mut self,
        player: Player,
        amount: i128,
        effects: &[LifeGainReplacementEffect],
    ) -> usize {
        if let Some(pick) = self.next_ordered_pick() {
            let Pick::LifeGain {
                player: chooser,
                amount: before,
                source,
                ability,
            } = pick
            else {
                unreachable!("exact life gain replay prefix");
            };
            assert_eq!(
                (chooser, before),
                (player, amount),
                "exact life gain replay prefix"
            );
            return effects
                .iter()
                .position(|effect| effect.source == source && effect.ability == ability)
                .expect("live life gain replacement");
        }
        std::panic::resume_unwind(Box::new(Request::LifeGain(LifeGainReplacementChoice {
            player,
            amount,
            effects: effects.to_vec(),
        })));
    }
    pub(super) fn freeze_zone_rules(
        &self,
        before: &ObjectView,
        requested: Zone,
        discard_cause: Option<Player>,
    ) -> FrozenRules {
        let prevent = self.zone_move_prevented(before.reference, requested);
        if requested != Zone::Graveyard {
            return FrozenRules {
                prevent,
                ..FrozenRules::default()
            };
        }
        let own = before.card.zone_replacement();
        let own_available =
            before.zone != Zone::Battlefield || self.printed_abilities_available(before.id);
        let card_or_token =
            !self.state.objects[before.id as usize].is_spell_copy() || before.is_token();
        let spell = before.kinds & 12 != 0 && !before.is_token() && card_or_token;
        let mut reflexive_exiles = Vec::new();
        let mut linked_death_exiles = Vec::new();
        if before.zone == Zone::Battlefield && before.is_creature() {
            for &id in &self.state.static_sources {
                let source = &self.state.objects[id as usize];
                let available = || {
                    source.zone == Zone::Battlefield
                        && source.controller != before.controller
                        && self.printed_abilities_available(id)
                };
                match source.card.zone_replacement() {
                    Some(ZoneReplacementRule::ExileOpposingCreatureDeath { ability })
                        if available() =>
                    {
                        reflexive_exiles.push((ability, self.object_view(id)));
                    }
                    Some(ZoneReplacementRule::ExileOpposingNontokenCreatureDeathLinked {
                        life,
                    }) if !before.is_token() && available() => {
                        let reference = self.object_ref(id);
                        linked_death_exiles.push(LinkedDeathExile {
                            source: reference,
                            ability_origin: self.copy_ability_origin(reference),
                            controller: source.controller,
                            life,
                        });
                    }
                    _ => {}
                }
            }
        }
        FrozenRules {
            dream: self.dream_resolution_mark(before.reference),
            prevent,
            reflexive_exiles,
            linked_death_exiles,
            shuffle_spell: false,
            shuffle_self: own_available && own == Some(ZoneReplacementRule::RevealShuffleSelf),
            exile_spell: (before.zone == Zone::Stack
                && self.stack_graveyard_exile(before.reference))
                || self.state.static_sources.iter().any(|&id| {
                    self.on_battlefield(id)
                        && self.printed_abilities_available(id)
                        && match self.state.objects[id as usize].card.zone_replacement() {
                            Some(ZoneReplacementRule::ExileAllCardsAndTokens) => card_or_token,
                            Some(ZoneReplacementRule::ExileControllerGraveyard) => {
                                card_or_token
                                    && !before.card.is_emblem()
                                    && self.state.objects[id as usize].controller == before.owner
                            }
                            Some(ZoneReplacementRule::ExileInstantSorceryCards) => spell,
                            Some(ZoneReplacementRule::ExileOpposingCreatureDeathPlain) => {
                                before.zone == Zone::Battlefield
                                    && before.is_creature()
                                    && self.state.objects[id as usize].controller
                                        != before.controller
                            }
                            _ => false,
                        }
                }),
            discard_to_battlefield: before.zone == Zone::Hand
                && own == Some(ZoneReplacementRule::OpponentDiscardToBattlefield)
                && discard_cause.is_some_and(|cause| cause != before.owner),
        }
    }
    /// CR 616.1: ask affected players in APNAP order. Retain the original
    /// movement order for commit, including explicit library positions.
    pub(super) fn plan_zone_batch<T>(
        &mut self,
        mut frozen: Vec<(ObjectView, Zone, FrozenRules, T)>,
    ) -> Vec<(ObjectView, Plan, T)> {
        let active = self.state.active;
        let requires_apnap = frozen.len() > 1
            && frozen.iter().any(|(_, _, rules, _)| {
                // All other effects collapse to a single exile option.
                rules.dream.is_some()
                    || rules.prevent
                    || rules.shuffle_self
                    || rules.shuffle_spell
                    || rules.discard_to_battlefield
                    || !rules.reflexive_exiles.is_empty()
                    || !rules.linked_death_exiles.is_empty()
            })
            && {
                let mut order = frozen
                    .iter()
                    .map(|(before, _, _, _)| affected_player(before));
                order.any(|player| player != active) && order.any(|player| player == active)
            };
        if requires_apnap {
            let mut plans = vec![None; frozen.len()];
            for player in [active, active.other()] {
                for ((before, requested, rules, _), plan) in frozen.iter_mut().zip(&mut plans) {
                    if affected_player(before) == player {
                        *plan =
                            Some(self.plan_zone_move(before, *requested, std::mem::take(rules)));
                    }
                }
            }
            return frozen
                .into_iter()
                .zip(plans)
                .map(|((before, _, _, data), plan)| {
                    (
                        before,
                        plan.expect("every affected player was considered"),
                        data,
                    )
                })
                .collect();
        }
        frozen
            .into_iter()
            .map(|(before, requested, rules, data)| {
                let plan = self.plan_zone_move(&before, requested, rules);
                (before, plan, data)
            })
            .collect()
    }

    pub(super) fn plan_zone_move(
        &mut self,
        before: &ObjectView,
        requested: Zone,
        rules: FrozenRules,
    ) -> Plan {
        if let Some(plan) = self.replay_frozen_mana_zone_plan(before.reference, requested) {
            return plan;
        }
        let plan = self.plan_zone_move_unrecorded(before, requested, rules);
        self.record_cast_mana_zone_plan(before, &plan);
        plan
    }
    fn plan_zone_move_unrecorded(
        &mut self,
        before: &ObjectView,
        requested: Zone,
        rules: FrozenRules,
    ) -> Plan {
        // A true prohibition stops the event before replacement selection; it
        // must never become a competing "prevent" replacement or a paid cost.
        if requested == Zone::Exile && self.graveyard_exile_prohibited(before.id) {
            return Plan {
                completion: None,
                destination: requested,
                prevented: true,
                shuffle: None,
                reflexive: None,
            };
        }
        #[cfg(feature = "test-support")]
        let test_redirect = requested == Zone::Exile
            && TEST_REDIRECT_EXILE_TO_BATTLEFIELD
                .with(|target| target.get() == Some(before.reference));
        #[cfg(not(feature = "test-support"))]
        let test_redirect = false;
        let exile_on_stack_exit =
            before.zone == Zone::Stack && self.state.objects[before.id as usize].flashback;
        if !rules.prevent && !test_redirect && requested != Zone::Graveyard && !exile_on_stack_exit
        {
            return Plan {
                completion: None,
                destination: requested,
                prevented: false,
                shuffle: None,
                reflexive: None,
            };
        }
        let mut choices = Vec::new();
        if let Some(mark) = rules.dream {
            choices.push(ZoneReplacementKind::DreamExile);
            match mark.face {
                CastFace::Adventure => choices.push(ZoneReplacementKind::AdventureExile),
                CastFace::Omen => choices.push(ZoneReplacementKind::OmenShuffle),
                _ => {}
            }
        }
        if rules.prevent {
            choices.push(ZoneReplacementKind::Prevent);
        }
        if test_redirect {
            choices.push(ZoneReplacementKind::Battlefield);
        }
        if exile_on_stack_exit
            || rules.exile_spell
            || self.death_exile_destination(before, requested) != requested
        {
            choices.push(ZoneReplacementKind::Exile);
        }
        for (ability, source) in &rules.reflexive_exiles {
            choices.push(ZoneReplacementKind::ExileReflexive {
                source: source.reference,
                ability: *ability,
            });
        }
        for replacement in &rules.linked_death_exiles {
            choices.push(ZoneReplacementKind::ExileLinked {
                source: replacement.source,
            });
        }
        if requested == Zone::Graveyard && rules.shuffle_self {
            choices.push(ZoneReplacementKind::RevealShuffle);
        }
        if requested == Zone::Graveyard && rules.shuffle_spell {
            choices.push(ZoneReplacementKind::Shuffle);
        }
        if requested == Zone::Graveyard && rules.discard_to_battlefield {
            choices.push(ZoneReplacementKind::Battlefield);
        }
        let effect = match choices.as_slice() {
            [] => {
                return Plan {
                    completion: None,
                    destination: requested,
                    prevented: false,
                    shuffle: None,
                    reflexive: None,
                };
            }
            [effect] => *effect,
            _ => {
                if let Some(pick) = self.next_ordered_pick() {
                    let Pick::Zone(pick) = pick else {
                        unreachable!("exact zone replay prefix");
                    };
                    assert_eq!(
                        (pick.object, pick.requested),
                        (before.reference, requested),
                        "exact zone replay prefix"
                    );
                    assert!(
                        choices.contains(&pick.effect),
                        "live zone replacement choice"
                    );
                    return Self::zone_plan(pick.effect, &rules, exile_on_stack_exit);
                }
                let request = Request::Zone(ZoneRequest {
                    object: before.reference,
                    requested,
                    affected: affected_player(before),
                    public: match before.zone {
                        Zone::Battlefield | Zone::Stack | Zone::Graveyard | Zone::Command => true,
                        Zone::Exile => !self.exiled_face_down(before.reference),
                        _ => false,
                    },
                    view: ZoneReplacementChoiceView {
                        card: before.card,
                        owner: before.owner,
                        from: before.zone,
                        choices,
                        prior_public_choices: Vec::new(),
                    },
                });
                // Private typed unwind uses the existing action checkpoint and never calls the panic hook.
                std::panic::resume_unwind(Box::new(request));
            }
        };
        Self::zone_plan(effect, &rules, exile_on_stack_exit)
    }
    fn zone_plan(
        effect: ZoneReplacementKind,
        rules: &FrozenRules,
        exile_on_stack_exit: bool,
    ) -> Plan {
        let shuffle = match effect {
            ZoneReplacementKind::Shuffle | ZoneReplacementKind::OmenShuffle => {
                Some(ZoneShuffle::Plain)
            }
            ZoneReplacementKind::RevealShuffle => Some(ZoneShuffle::Reveal),
            _ => None,
        };
        Plan {
            completion: match effect {
                ZoneReplacementKind::DreamExile => Some(ZoneMoveCompletion::Dream),
                ZoneReplacementKind::AdventureExile => Some(ZoneMoveCompletion::Adventure(
                    rules.dream.expect("marked adventure").player,
                )),
                ZoneReplacementKind::ExileLinked { source } => {
                    Some(ZoneMoveCompletion::LinkedDeathExile(
                        *rules
                            .linked_death_exiles
                            .iter()
                            .find(|replacement| replacement.source == source)
                            .expect("frozen linked replacement source"),
                    ))
                }
                _ => None,
            },
            destination: match effect {
                ZoneReplacementKind::Prevent => Zone::Graveyard,
                ZoneReplacementKind::Shuffle
                | ZoneReplacementKind::RevealShuffle
                | ZoneReplacementKind::OmenShuffle => {
                    // An exile-on-stack-exit replacement still applies to the
                    // move within the shuffle. CR 701.24c retains the shuffle.
                    if exile_on_stack_exit {
                        Zone::Exile
                    } else {
                        Zone::Library
                    }
                }
                ZoneReplacementKind::DreamExile
                | ZoneReplacementKind::AdventureExile
                | ZoneReplacementKind::Exile
                | ZoneReplacementKind::ExileReflexive { .. }
                | ZoneReplacementKind::ExileLinked { .. } => Zone::Exile,
                ZoneReplacementKind::Battlefield => Zone::Battlefield,
            },
            prevented: effect == ZoneReplacementKind::Prevent,
            shuffle,
            reflexive: if let ZoneReplacementKind::ExileReflexive { source, ability } = effect {
                Some((
                    ability,
                    Box::new(
                        rules
                            .reflexive_exiles
                            .iter()
                            .find(|(index, view)| *index == ability && view.reference == source)
                            .expect("frozen selected replacement source")
                            .1
                            .clone(),
                    ),
                ))
            } else {
                None
            },
        }
    }
    pub(super) fn shuffle_zone_card(&mut self, before: &ObjectView, shuffle: ZoneShuffle) {
        if shuffle == ZoneShuffle::Reveal {
            self.record_zone_reveal(before.card, before.owner, before.zone);
        }
        self.forget_library_top(before.owner);
        self.shuffle_library_cards(before.owner);
    }
    pub(super) fn suspend_zone_replacement(
        &mut self,
        request: Request,
        checkpoint: State,
        actor: Player,
        action: Option<Action>,
        revision: u64,
    ) -> Result<(), Error> {
        let Some(next_revision) = revision.checked_add(1) else {
            self.state = checkpoint;
            self.revision = revision;
            return Err(Error::CapacityExceeded(crate::capacity::CapacityExceeded {
                resource: "revision exhausted",
            }));
        };
        let replay = self.state.zone_rules.as_mut().and_then(|r| r.replay.take());
        let base = if let Some(replay) = replay {
            replay.base
        } else {
            let current = std::mem::replace(&mut self.state, checkpoint);
            let bytes = self.state_bytes();
            let state = std::mem::replace(&mut self.state, current);
            assert!(
                state
                    .zone_rules
                    .as_ref()
                    .is_none_or(|r| r.choice.is_none() && r.replay.is_none()),
                "nonrecursive zone checkpoint"
            );
            Base {
                state: Box::new(state),
                bytes,
                actor,
                action: action.expect("zone-aware action"),
                picks: SharedVec::default(),
            }
        };
        self.state.pending = match &request {
            Request::Zone(zone) => Pending::ZoneReplacement {
                player: zone.affected,
            },
            Request::LifeGain(life) => Pending::LifeGainReplacement {
                player: life.player,
            },
        };
        self.state.zone_rules.get_or_insert_default().choice =
            Some(Box::new(Choice { base, request }));
        self.revision = next_revision;
        Ok(())
    }
    pub(super) fn choose_zone_replacement(&mut self, effect: ZoneReplacementKind) {
        self.choose_ordered_event(Selection::Zone(effect));
    }
    pub(super) fn choose_life_gain_replacement(&mut self, source: ObjectRef, ability: u16) {
        self.choose_ordered_event(Selection::LifeGain { source, ability });
    }
    fn choose_ordered_event(&mut self, selection: Selection) {
        let foreign_cast_choice = self.foreign_cast_choice_pending();
        let choice = self
            .state
            .zone_rules
            .as_mut()
            .unwrap()
            .choice
            .take()
            .unwrap();
        let mut base = choice.base;
        base.picks.push(match (choice.request, selection) {
            (Request::Zone(request), Selection::Zone(effect)) => Pick::Zone(ZonePick {
                object: request.object,
                requested: request.requested,
                effect,
                player: request.affected,
                public: request.public,
            }),
            (Request::LifeGain(request), Selection::LifeGain { source, ability }) => {
                Pick::LifeGain {
                    player: request.player,
                    amount: request.amount,
                    source,
                    ability,
                }
            }
            _ => unreachable!("choice for the active event"),
        });
        let actor = base.actor;
        let action = base.action.clone();
        let revision = self.revision;
        let mana_replay = self.state.mana_lots.as_mut().and_then(|r| r.replay.take());
        self.state = (*base.state).clone();
        if let Some(mana_replay) = mana_replay {
            self.state.mana_lots.get_or_insert_default().replay = Some(mana_replay);
        }
        self.state.zone_rules.get_or_insert_default().replay =
            Some(Box::new(Replay { base, next: 0 }));
        let result = if matches!(self.state.pending, Pending::ManaLotSpend { .. }) {
            self.apply_mana_lot_choice(actor, action)
        } else {
            self.apply_transaction(actor, action)
        };
        result.expect("saved legal zone action remains legal");
        let replay = self
            .state
            .zone_rules
            .as_mut()
            .unwrap()
            .replay
            .take()
            .unwrap();
        assert_eq!(
            replay.next,
            replay.base.picks.len(),
            "complete zone replay prefix"
        );
        self.revision = revision;
        // The saved action runs as its original actor. Retain the actual foreign
        // replacement decision only after replay bookkeeping has been consumed.
        if foreign_cast_choice {
            self.mark_foreign_cast_choice();
        }
    }
}

/// A failed casting proposal cannot conceal a real irreversible library reveal.
/// The choice/replay stack and stack-to-graveyard permissions remain checkpoint-owned.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn preserve_irreversible_reveals(
    target: &mut Option<Box<ZoneState>>,
    source: Option<&ZoneState>,
) {
    if let Some(source) = source {
        if !source.reveals.is_empty() || target.is_some() {
            target.get_or_insert_default().reveals = source.reveals.clone();
        }
    } else if let Some(target) = target {
        target.reveals.clear();
    }
}

/// Preserve only the currently live replay when restoring a staged cost.
/// A checkpoint can outlive the replay that created it, or can fail while
/// that replay is still on the caller's stack.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn transfer_live_replay(
    from: &mut Option<Box<ZoneState>>,
    to: &mut Option<Box<ZoneState>>,
) {
    let live = from.as_mut().and_then(|rules| rules.replay.take());
    if live.is_some() || to.is_some() {
        to.get_or_insert_default().replay = live;
    }
}
