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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub copy_recipes: Vec<Option<CopyRecipe>>,
    /// Parallel to tokens for Role events; None denotes an unbound replacement addition.
    /// Ordinary token and counter events use an empty vector.
    pub role_attachments: Vec<Option<ObjectRef>>,
    pub effects: Vec<ReplacementEffectView>,
    /// Current public damage event, before the next chosen replacement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub damage: Option<DamageEvent>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CounterPlacement {
    pub object: ObjectRef,
    pub placed_by: Player,
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
    TemporaryCopy(Box<SpellView>),
    Keywords {
        source: ObjectRef,
        keywords: Keywords,
    },
    Reflexive {
        item: Box<SpellView>,
        ability: u16,
    },
    JobAttach(ObjectRef),
    RoleAttach(ObjectRef),
    RoleReflexive(Box<RoleReflexiveCompletion>),
    /// One mass creation event; original group indices retain exact recipients.
    RoleAttachMany(Vec<ObjectRef>),
    Mobilize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct TokenBatch {
    recipes: Vec<Option<CopyRecipe>>,
    player: Player,
    groups: Vec<TokenGroupView>,
    effect: bool,
    used: Vec<(ObjectRef, u16)>,
    completion: TokenCompletion,
}
impl TokenBatch {
    fn artifact_at(&self, index: usize) -> bool {
        self.groups[index].token.is_artifact()
            || self
                .recipes
                .get(index)
                .and_then(|r| *r)
                .is_some_and(|r| r.exceptions.artifact || r.exceptions.golem)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct DamageBatch {
    events: Vec<DamageEvent>,
    next: usize,
    used: Vec<(ObjectRef, u16)>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum Event {
    Damage(DamageBatch),
    Counters(CounterBatch),
    Tokens(TokenBatch),
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum Work {
    Damage(DamageBatch),
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
                + b.placements
                    .iter()
                    .map(|p| p.amount.owned_bytes())
                    .sum::<usize>()
                + b.used.len() * std::mem::size_of::<(ObjectRef, u16)>()
        };
        let token_bytes = |b: &TokenBatch| {
            b.groups.len() * std::mem::size_of::<TokenGroupView>()
                + b.recipes.len() * std::mem::size_of::<Option<CopyRecipe>>()
                + b.used.len() * std::mem::size_of::<(ObjectRef, u16)>()
                + match &b.completion {
                    TokenCompletion::TemporaryCopy(item)
                    | TokenCompletion::Reflexive { item, .. } => {
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
                    Work::Damage(b) => {
                        b.events.len() * std::mem::size_of::<DamageEvent>()
                            + b.used.len() * std::mem::size_of::<(ObjectRef, u16)>()
                    }
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
                    Event::Damage(b) => {
                        b.events.len() * std::mem::size_of::<DamageEvent>()
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
fn matches_filter(filter: ReplacementFilter, p: &CounterPlacement) -> bool {
    let subtype = |s| p.subtypes.has(p.card, s);
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
        && kind.map_or_else(|| p.amount.has_any(), |k| p.amount.get(k) > 0)
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
                    && self.active_rule_fragment(id, |f| {
                        matches!(f, crate::cards::StaticAbilityFragment::PlayerHexproof)
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
                    ReplacementSpec::PreventAllDamageToSelf => {
                        event.preventable
                            && matches!(event.target, Target::Card(target) if self.on_battlefield(target) && self.object_ref(target) == source)
                    }
                    ReplacementSpec::PreventOtherCreatureNoncombatDamage => {
                        event.preventable
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
    fn modify_damage(choice: ReplacementEffectView, event: &mut DamageEvent) {
        match choice.source_card.replacement_effects()[usize::from(choice.ability)] {
            ReplacementSpec::DoubleCreatureDamage | ReplacementSpec::DoubleOpponentDamage => {
                event.amount = event
                    .amount
                    .checked_mul(2)
                    .capacity("damage replacement quantity exhausted")
            }
            ReplacementSpec::PreventOtherCreatureNoncombatDamage
            | ReplacementSpec::PreventAllDamageToSelf => event.amount = 0,
            _ => unreachable!(),
        }
    }
    pub(super) fn damage_replacement_candidates_exist(&self, events: &[DamageEvent]) -> bool {
        events
            .iter()
            .any(|event| !self.damage_replacements(event, &[]).is_empty())
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
            next: 0,
            used: Vec::new(),
        }));
    }
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
                    .counters = p.amount.clone();
            } else if self.on_battlefield(p.object.card)
                && self.object_ref(p.object.card) == p.object
            {
                let before = self.state.objects[p.object.card as usize].counters.lore;
                for kind in CounterKind::ALL {
                    self.add_counter(p.object.card, kind, p.amount.get(kind));
                }
                for kind in CounterKind::ALL {
                    self.self_counters_added(p.object, kind, p.amount.get(kind), p.placed_by);
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
        placements.retain(|p| p.amount.has_any());
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
    pub(super) fn activation_cost_counters(
        &mut self,
        object: ObjectRef,
        kind: CounterKind,
        amount: u64,
        player: Player,
    ) {
        let o = &self.state.objects[object.card as usize];
        let placement = CounterPlacement {
            placed_by: player,
            object,
            card: o.card,
            owner: o.owner,
            controller: o.controller,
            subtypes: self.granted_subtypes(object.card),
            kind: self.current_kinds(object.card),
            amount: Counters::of(kind, amount),
            effect: false,
            entry_index: None,
            private: false,
        };
        self.stage_counter_placements(vec![placement], ResumeMode::Pending);
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
                    placed_by: self
                        .state
                        .resolving
                        .as_ref()
                        .map_or(o.controller, |r| r.item.controller),
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
            self.add_counter(id, kind, amount);
            let placed_by = self
                .state
                .resolving
                .as_ref()
                .map_or(self.state.objects[id as usize].controller, |r| {
                    r.item.controller
                });
            self.self_counters_added(self.object_ref(id), kind, amount, placed_by);
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
    fn stage_copy_creation_inner(
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
        if self.state.replacements.is_none() && self.token_replacements(&batch).is_empty() {
            let temporary = match &batch.completion {
                TokenCompletion::TemporaryCopy(item) => Some(item.as_ref()),
                _ => None,
            };
            self.create_copy_groups_inner(player, &batch.groups, &batch.recipes, temporary);
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
                                if !self.keywords(id).contains(Keywords::INDESTRUCTIBLE) {
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
                if resume_program
                    && state.mode == ResumeMode::Program
                    && self.state.resolving.is_some()
                {
                    self.run_program();
                }
                return self.state.replacements.is_some();
            };
            match active {
                Event::Damage(batch) => {
                    if batch.next == batch.events.len() {
                        let Some(Event::Damage(batch)) = state.active.take() else {
                            unreachable!()
                        };
                        self.commit_damage_batch(&batch.events);
                        continue;
                    }
                    let event = batch.events[batch.next];
                    let choices = self.damage_replacements(&event, &batch.used);
                    match choices.as_slice() {
                        [] => {
                            batch.next += 1;
                            batch.used.clear();
                        }
                        [only] => {
                            Self::modify_damage(*only, &mut batch.events[batch.next]);
                            batch.used.push((only.source, only.ability));
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
                            if !batch.recipes.is_empty() {
                                let temporary = match &batch.completion {
                                    TokenCompletion::TemporaryCopy(item) => Some(item.as_ref()),
                                    _ => None,
                                };
                                self.create_copy_groups_inner(
                                    batch.player,
                                    &batch.groups,
                                    &batch.recipes,
                                    temporary,
                                );
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
                                    TokenCompletion::JobAttach(_)
                                        | TokenCompletion::Keywords { .. }
                                        | TokenCompletion::Reflexive { .. }
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
            Event::Damage(batch) => {
                let choice = self
                    .damage_replacements(&batch.events[batch.next], &batch.used)
                    .into_iter()
                    .find(|c| c.source == source && c.ability == ability)
                    .unwrap();
                Self::modify_damage(choice, &mut batch.events[batch.next]);
                batch.used.push((source, ability));
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
            Event::Damage(b) => self.damage_replacements(&b.events[b.next], &b.used),
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
            Event::Damage(batch) => Some(ReplacementChoiceView {
                kind: "Damage",
                object: None,
                counters: None,
                tokens: Vec::new(),
                copy_recipes: Vec::new(),
                role_attachments: Vec::new(),
                effects: self.damage_replacements(&batch.events[batch.next], &batch.used),
                damage: Some(batch.events[batch.next]),
            }),
            Event::Counters(batch) => {
                let p = &batch.placements[batch.next];
                if p.private && viewer != p.owner {
                    return None;
                }
                Some(ReplacementChoiceView {
                    damage: None,
                    kind: "Counters",
                    object: Some(p.object),
                    counters: Some(p.amount.clone()),
                    tokens: Vec::new(),
                    copy_recipes: Vec::new(),
                    role_attachments: Vec::new(),
                    effects: self.counter_replacements(p, &batch.used),
                })
            }
            Event::Tokens(batch) => Some(ReplacementChoiceView {
                damage: None,
                kind: "Tokens",
                object: None,
                counters: None,
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
#[derive(Clone, Debug, Serialize)]
pub struct CreatedTokenChoiceView {
    pub source: ObjectRef,
    pub candidates: Vec<ObjectRef>,
}
