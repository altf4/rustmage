//! Owned, event-local replacement ordering. Immutable programs stay in the catalogue.
use super::*;
use crate::cards::{ReplacementFilter, ReplacementSpec};
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ResumeMode {
    Program,
    Pending,
    Entry,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ReplacementEffectView {
    pub source: ObjectRef,
    pub source_card: Card,
    pub ability: u16,
}
#[derive(Clone, Debug, Serialize)]
pub struct ReplacementChoiceView {
    pub kind: &'static str,
    pub object: Option<ObjectRef>,
    pub counters: Option<Counters>,
    pub tokens: Vec<TokenGroupView>,
    /// Parallel to tokens for Role events; None denotes an unbound replacement addition.
    /// Ordinary token and counter events use an empty vector.
    pub role_attachments: Vec<Option<ObjectRef>>,
    pub effects: Vec<ReplacementEffectView>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CounterPlacement {
    pub object: ObjectRef,
    pub card: Card,
    pub owner: Player,
    pub controller: Player,
    pub subtypes: GrantedSubtypes,
    pub kind: u16,
    pub amount: Counters,
    pub effect: bool,
    pub entry_index: Option<usize>,
    pub private: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct CounterBatch {
    placements: Vec<CounterPlacement>,
    next: usize,
    used: Vec<(ObjectRef, u16)>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct TokenGroupView {
    pub token: Card,
    pub amount: u128,
    pub tapped: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RoleReflexiveCompletion {
    pub recipient: ObjectRef,
    pub token: Card,
    pub item: SpellView,
    pub ability: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum TokenCompletion {
    None,
    JobAttach(ObjectRef),
    RoleAttach(ObjectRef),
    RoleReflexive(Box<RoleReflexiveCompletion>),
    /// One mass creation event; original group indices retain exact recipients.
    RoleAttachMany(Vec<ObjectRef>),
    Mobilize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct TokenBatch {
    player: Player,
    groups: Vec<TokenGroupView>,
    effect: bool,
    used: Vec<(ObjectRef, u16)>,
    completion: TokenCompletion,
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum Event {
    Counters(CounterBatch),
    Tokens(TokenBatch),
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum Work {
    Counters(CounterBatch),
    Tokens(TokenBatch),
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReplacementState {
    work: VecDeque<Work>,
    active: Option<Event>,
    attachment: Option<(ObjectRef, Vec<ObjectRef>)>,
    resume: Option<Pending>,
    mode: ResumeMode,
}
impl ReplacementState {
    pub fn bytes(&self) -> usize {
        let counter_bytes = |b: &CounterBatch| {
            b.placements.len() * std::mem::size_of::<CounterPlacement>()
                + b.used.len() * std::mem::size_of::<(ObjectRef, u16)>()
        };
        let token_bytes = |b: &TokenBatch| {
            b.groups.len() * std::mem::size_of::<TokenGroupView>()
                + b.used.len() * std::mem::size_of::<(ObjectRef, u16)>()
                + match &b.completion {
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
                    Work::Counters(b) => counter_bytes(b),
                    Work::Tokens(b) => token_bytes(b),
                    Work::DestroyAndCreate { targets, .. } => {
                        targets.len() * std::mem::size_of::<BoundTarget>()
                    }
                    _ => 0,
                })
                .sum::<usize>()
            + self.active.as_ref().map_or(0, |b| {
                // The inline Option<Event> storage is already in size_of::<Self>().
                match b {
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
fn matches_filter(filter: ReplacementFilter, p: &CounterPlacement) -> bool {
    let subtype = |s| p.card.has_subtype(s) || p.subtypes.contains(s);
    match filter {
        ReplacementFilter::Permanent => true,
        ReplacementFilter::Creature => p.kind & 1 != 0,
        ReplacementFilter::CreatureOrVehicle => p.kind & 1 != 0 || subtype("Vehicle"),
        ReplacementFilter::CreatureSpacecraftPlanet => {
            p.kind & 1 != 0 || subtype("Spacecraft") || subtype("Planet")
        }
    }
}
fn counter_applies(spec: ReplacementSpec, p: &CounterPlacement) -> bool {
    let ReplacementSpec::Counters {
        filter,
        kind,
        effects_only,
        ..
    } = spec
    else {
        return false;
    };
    matches_filter(filter, p)
        && (!effects_only || p.effect)
        && kind.map_or_else(|| p.amount.total() > 0, |k| p.amount.get(k) > 0)
}
fn modify_counter(spec: ReplacementSpec, p: &mut CounterPlacement) {
    let ReplacementSpec::Counters {
        kind, factor, add, ..
    } = spec
    else {
        unreachable!()
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
impl Game {
    fn counter_replacements(
        &self,
        p: &CounterPlacement,
        used: &[(ObjectRef, u16)],
    ) -> Vec<ReplacementEffectView> {
        let mut out = Vec::new();
        let mut add = |reference: ObjectRef, card: Card, controller: Player| {
            if controller != p.controller {
                return;
            }
            for (index, &spec) in card.replacement_effects().iter().enumerate() {
                if self.on_battlefield(reference.card) && self.object_ref(reference.card) == reference
                    && !self.continuous_fragment_available(reference.card, |fragment| matches!(fragment,
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
        for &id in &self.state.static_sources {
            let o = &self.state.objects[id as usize];
            if o.zone == Zone::Battlefield {
                add(self.object_ref(id), o.card, o.controller);
            }
        }
        // CR614.12: only the incoming permanent's own effects may affect its entry;
        // a different member of this simultaneous entry batch is not yet a source.
        if p.entry_index.is_some() {
            add(p.object, p.card, p.controller);
        }
        out
    }
    fn commit_counter_placements(&mut self, placements: Vec<CounterPlacement>) {
        let mut lore = Vec::new();
        for p in placements {
            if let Some(index) = p.entry_index {
                self.state
                    .entry_batch
                    .as_mut()
                    .expect("owned entry counter batch")
                    .moves[index]
                    .counters = p.amount;
            } else if self.on_battlefield(p.object.card)
                && self.object_ref(p.object.card) == p.object
            {
                let before = self.state.objects[p.object.card as usize].counters.lore;
                for kind in CounterKind::ALL {
                    self.state.objects[p.object.card as usize]
                        .counters
                        .add(kind, p.amount.get(kind));
                }
                if p.amount.lore > 0 {
                    lore.push((p.object, before));
                }
            }
        }
        for (object, before) in lore {
            self.lore_counters_added(object, before);
        }
    }
    pub(super) fn stage_counter_placements(
        &mut self,
        mut placements: Vec<CounterPlacement>,
        mode: ResumeMode,
    ) {
        placements.retain(|p| p.amount.total() > 0);
        if placements.is_empty() {
            return;
        }
        placements.sort_by_key(|p| (p.controller != self.state.active, p.object.card));
        if self.state.replacements.is_none()
            && placements
                .iter()
                .all(|p| self.counter_replacements(p, &[]).is_empty())
        {
            self.commit_counter_placements(placements);
            return;
        }
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
        state.work.push_back(Work::Counters(CounterBatch {
            placements,
            next: 0,
            used: Vec::new(),
        }));
    }
    pub(super) fn effect_counters(
        &mut self,
        changes: Vec<(ObjectRef, Counters)>,
        mode: ResumeMode,
    ) {
        let placements = changes
            .into_iter()
            .filter_map(|(object, amount)| {
                if !self.on_battlefield(object.card) || self.object_ref(object.card) != object {
                    return None;
                }
                let o = &self.state.objects[object.card as usize];
                Some(CounterPlacement {
                    object,
                    card: o.card,
                    owner: o.owner,
                    controller: o.controller,
                    subtypes: self.granted_subtypes(object.card),
                    kind: self.current_kinds(object.card),
                    amount,
                    effect: true,
                    entry_index: None,
                    private: false,
                })
            })
            .collect();
        self.stage_counter_placements(placements, mode);
    }
    pub(super) fn effect_counter(
        &mut self,
        id: CardId,
        kind: CounterKind,
        amount: u64,
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
                    .any(|e| matches!(e, ReplacementSpec::Counters { .. }))
            })
        {
            let before = self.state.objects[id as usize].counters.lore;
            self.state.objects[id as usize].counters.add(kind, amount);
            if kind == CounterKind::Lore {
                self.lore_counters_added(self.object_ref(id), before);
            }
            return;
        }
        let mut counters = Counters::default();
        counters.add(kind, amount);
        self.effect_counters(vec![(self.object_ref(id), counters)], mode);
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
                    ReplacementSpec::ReplaceArtifactTokens(_) => b
                        .groups
                        .iter()
                        .any(|g| g.amount > 0 && g.token.is_artifact()),
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
            ReplacementSpec::AddToken(token) => {
                // The added token inherits the event's tapped/attacking instructions.
                let tapped = b.groups[0].tapped;
                if let Some(g) = b
                    .groups
                    .iter_mut()
                    .find(|g| g.token == token && g.tapped == tapped)
                {
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
                }
            }
            ReplacementSpec::ReplaceArtifactTokens(token) => {
                for g in &mut b.groups {
                    if g.token.is_artifact() {
                        g.token = token;
                    }
                }
            }
            _ => unreachable!(),
        }
        b.effect = true;
        b.used.push((choice.source, choice.ability));
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
                            ReplacementSpec::DoubleTokens { .. } | ReplacementSpec::AddToken(_)
                        ) || matches!(s, ReplacementSpec::ReplaceArtifactTokens(_))
                            && token.is_artifact()
                    })
            })
        {
            self.create_token_batch_capture(player, token, amount, tapped, None, false);
            return;
        }

        let batch = TokenBatch {
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
                                if !self.keywords(id).contains(Keywords::INDESTRUCTIBLE) {
                                    self.move_target_batch(&[bound], Zone::Graveyard);
                                }
                                state.work.push_front(Work::Tokens(TokenBatch {
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
                if resume_program
                    && state.mode == ResumeMode::Program
                    && self.state.resolving.is_some()
                {
                    self.run_program();
                }
                return self.state.replacements.is_some();
            };
            match active {
                Event::Counters(batch) => {
                    if batch.next == batch.placements.len() {
                        let Some(Event::Counters(batch)) = state.active.take() else {
                            unreachable!()
                        };
                        self.commit_counter_placements(batch.placements);
                        continue;
                    }
                    let p = &batch.placements[batch.next];
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
                            if matches!(batch.completion, TokenCompletion::Mobilize) {
                                self.start_mobilize_groups(batch.groups);
                            } else {
                                let capture =
                                    matches!(batch.completion, TokenCompletion::JobAttach(_));
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
                                if let TokenCompletion::RoleReflexive(c) = &batch.completion
                                    && created.iter().any(|r| {
                                        self.object_view(r.card).card == c.token
                                            && self.object_view(r.card).attached_to
                                                == Some(c.recipient)
                                    })
                                {
                                    self.queue_reflexive(&c.item, c.ability, Some(c.recipient));
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
    pub(super) fn choose_replacement(&mut self, source: ObjectRef, ability: u16) {
        let mut state = self.state.replacements.take().unwrap();
        match state.active.as_mut().unwrap() {
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
    pub(super) fn visit_replacement_actions(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let active = self
            .state
            .replacements
            .as_ref()
            .unwrap()
            .active
            .as_ref()
            .unwrap();
        let choices = match active {
            Event::Counters(b) => self.counter_replacements(&b.placements[b.next], &b.used),
            Event::Tokens(b) => self.token_replacements(b),
        };
        choices.into_iter().any(|c| {
            emit(Action::ChooseReplacement {
                source: c.source,
                ability: c.ability,
            })
        })
    }
    pub(super) fn replacement_choice(&self, viewer: Player) -> Option<ReplacementChoiceView> {
        if !matches!(self.state.pending, Pending::Replacement { .. }) {
            return None;
        }
        match self.state.replacements.as_ref()?.active.as_ref()? {
            Event::Counters(batch) => {
                let p = &batch.placements[batch.next];
                if p.private && viewer != p.owner {
                    return None;
                }
                Some(ReplacementChoiceView {
                    kind: "Counters",
                    object: Some(p.object),
                    counters: Some(p.amount),
                    tokens: Vec::new(),
                    role_attachments: Vec::new(),
                    effects: self.counter_replacements(p, &batch.used),
                })
            }
            Event::Tokens(batch) => Some(ReplacementChoiceView {
                kind: "Tokens",
                object: None,
                counters: None,
                tokens: batch.groups.clone(),
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
#[derive(Clone, Debug, Serialize)]
pub struct CreatedTokenChoiceView {
    pub source: ObjectRef,
    pub candidates: Vec<ObjectRef>,
}

#[cfg(test)]
mod owned_memory_tests {
    use super::*;
    #[test]
    fn inline_active_storage_is_counted_once_and_nested_owned_payloads_remain_counted() {
        let mut frame = ReplacementState {
            work: VecDeque::new(),
            active: None,
            attachment: None,
            resume: None,
            mode: ResumeMode::Program,
        };
        let empty = frame.bytes();
        frame.active = Some(Event::Tokens(TokenBatch {
            player: Player::P0,
            groups: Vec::new(),
            effect: true,
            used: Vec::new(),
            completion: TokenCompletion::None,
        }));
        assert_eq!(
            frame.bytes(),
            empty,
            "activating empty inline storage allocates no owned payload"
        );
        let Some(Event::Tokens(batch)) = frame.active.as_mut() else {
            unreachable!()
        };
        batch.groups.push(TokenGroupView {
            token: Card::Forest,
            amount: 1,
            tapped: false,
        });
        batch.used.push((
            ObjectRef {
                card: 0,
                generation: 0,
            },
            0,
        ));
        let payload =
            std::mem::size_of::<TokenGroupView>() + std::mem::size_of::<(ObjectRef, u16)>();
        assert_eq!(frame.bytes(), empty + payload);
        let mut entry = entries::EntryBatch {
            moves: Vec::new(),
            choices: Vec::new(),
            next: 0,
            counter_next: 0,
            counter_order: Vec::new(),
            timestamp_order: Vec::new(),
            timestamps_ready: false,
            resume: None,
            shuffle_after: None,
            replacement_resume: None,
        };
        let without_outer = entry.bytes();
        entry.replacement_resume = Some(Box::new(frame.clone()));
        assert_eq!(entry.bytes(), without_outer + frame.bytes());
        assert_eq!(entry.clone().bytes(), entry.bytes());
    }
}

#[cfg(test)]
mod role_success_memory_tests {
    use super::*;
    #[test]
    fn boxed_role_success_packet_and_nested_subject_are_counted_exactly_once() {
        let mut g = super::super::tests::scenario();
        let id = super::super::tests::spawn(
            &mut g,
            Player::P0,
            Card::from_name("Curse of the Werefox").unwrap(),
            Zone::Hand,
        );
        let item = g.cast_item(Player::P0, id, [0; 6]);
        let mut state = ReplacementState {
            work: VecDeque::new(),
            active: None,
            attachment: None,
            resume: None,
            mode: ResumeMode::Program,
        };
        state.active = Some(Event::Tokens(TokenBatch {
            player: Player::P0,
            groups: Vec::new(),
            effect: true,
            used: Vec::new(),
            completion: TokenCompletion::None,
        }));
        let before = state.bytes();
        let recipient = g.object_ref(id);
        let Some(Event::Tokens(batch)) = &mut state.active else {
            unreachable!()
        };
        batch.completion = TokenCompletion::RoleReflexive(Box::new(RoleReflexiveCompletion {
            recipient,
            token: Card::from_name("Curse of the Werefox").unwrap(),
            item: item.clone(),
            ability: 0,
        }));
        assert_eq!(
            state.bytes() - before,
            std::mem::size_of::<RoleReflexiveCompletion>() + item.owned_bytes()
        );
        let n = state.bytes();
        let Some(Event::Tokens(batch)) = &mut state.active else {
            unreachable!()
        };
        let TokenCompletion::RoleReflexive(c) = &mut batch.completion else {
            unreachable!()
        };
        c.item.reflexive_subject = Some(Box::new(ReflexiveSubjectView { object: recipient }));
        assert_eq!(
            state.bytes() - n,
            std::mem::size_of::<ReflexiveSubjectView>()
        );
    }
}
