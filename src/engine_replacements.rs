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
    AttackingCopies,
    MobilizeDelayed(Box<SpellView>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct TokenBatch {
    first_this_turn: bool,
    recipes: Vec<Option<CopyRecipe>>,
    player: Player,
    groups: Vec<TokenGroupView>,
    effect: bool,
    used: Vec<(ObjectRef, u16)>,
    completion: TokenCompletion,
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
    /// Damage results wait until their wither counter replacements are complete.
    DamageResults(Vec<DamageEvent>),
    Counters(CounterBatch),
    Tokens(TokenBatch),
    CompleteTokens {
        player: Player,
        completion: TokenCompletion,
        created: Vec<ObjectRef>,
    },
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
    optional_token: Option<ReplacementEffectView>,
    resume: Option<Pending>,
    mode: ResumeMode,
}
impl ReplacementState {
    pub(super) fn visit_owned_packets(&mut self, visit: &mut impl FnMut(&mut SpellView)) {
        if let Some(Event::Tokens(batch)) = &mut self.active {
            batch.completion.visit_owned_packets(visit);
        }
        for work in &mut self.work {
            match work {
                Work::Tokens(batch) => batch.completion.visit_owned_packets(visit),
                Work::CompleteTokens { completion, .. } => completion.visit_owned_packets(visit),
                _ => {}
            }
        }
    }
}
impl TokenCompletion {
    fn visit_owned_packets(&mut self, visit: &mut impl FnMut(&mut SpellView)) {
        match self {
            Self::Reflexive { item, .. } | Self::MobilizeDelayed(item) => visit(item),
            Self::RoleReflexive(completion) => visit(&mut completion.item),
            _ => {}
        }
    }
    fn owned_bytes(&self) -> usize {
        match self {
            Self::Reflexive { item, .. } | Self::MobilizeDelayed(item) => {
                std::mem::size_of::<SpellView>() + item.owned_bytes()
            }
            Self::RoleReflexive(c) => {
                std::mem::size_of::<RoleReflexiveCompletion>() + c.item.owned_bytes()
            }
            Self::RoleAttachMany(r) => r.len() * std::mem::size_of::<ObjectRef>(),
            _ => 0,
        }
    }
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
                + b.completion.owned_bytes()
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
                    Work::DamageResults(events) => {
                        events.len() * std::mem::size_of::<DamageEvent>()
                    }
                    Work::Counters(b) => counter_bytes(b),
                    Work::Tokens(b) => token_bytes(b),
                    Work::CompleteTokens {
                        completion,
                        created,
                        ..
                    } => {
                        completion.owned_bytes() + created.len() * std::mem::size_of::<ObjectRef>()
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
                    ReplacementSpec::DoubleControlledChosenTypeDamage => {
                        event.source.controller == object.controller
                            && match self.entry_value(source) {
                                Some(crate::cards::EntryChoiceValue::CreatureType(kind)) => {
                                    event.source.facts.has_subtype(kind.name())
                                }
                                _ => false,
                            }
                    }
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
        if event.preventable
            && let Target::Player(player) = event.target
            && let Some(shields) = &self.state.player_damage_shields
        {
            for shield in shields.iter().filter(|row| row.controller == player) {
                if !used.contains(&(shield.source, u16::MAX - 1)) {
                    out.push(ReplacementEffectView {
                        source: shield.source,
                        source_card: shield.source_card,
                        ability: u16::MAX - 1,
                    });
                }
            }
        }
        if event.preventable
            && let Target::Card(id) = event.target
            && self.on_battlefield(id)
            && ((event.source.facts.colors != 0
                && self.keywords(id).contains(Keywords::PROTECTION_COLORS))
                || (event.source.controller != self.state.objects[id as usize].controller
                    && self.keywords(id).contains(Keywords::PROTECTION_OPPONENTS)))
        {
            let source = self.object_ref(id);
            if !used.contains(&(source, u16::MAX)) {
                out.push(ReplacementEffectView {
                    source,
                    source_card: self.state.objects[id as usize].card,
                    ability: u16::MAX,
                });
            }
        }
        out
    }
    fn modify_damage(choice: ReplacementEffectView, event: &mut DamageEvent) {
        // Reserved dynamic protection identity; it has no immutable card-array slot.
        if choice.ability == u16::MAX || choice.ability == u16::MAX - 1 {
            event.amount = 0;
            return;
        }
        match choice.source_card.replacement_effects()[usize::from(choice.ability)] {
            ReplacementSpec::DoubleCreatureDamage
            | ReplacementSpec::DoubleOpponentDamage
            | ReplacementSpec::DoubleControlledChosenTypeDamage => {
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
                optional_token: None,
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
                && self.can_receive_counters(p.object.card)
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
                optional_token: None,
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
    fn wither_damage_counters(&self, events: &[DamageEvent]) -> Vec<CounterPlacement> {
        events
            .iter()
            .filter_map(|event| {
                if event.amount <= 0 || !event.source.keywords.contains(Keywords::WITHER) {
                    return None;
                }
                let Target::Card(id) = event.target else {
                    return None;
                };
                if !self.on_battlefield(id) || !self.is_creature(id) {
                    return None;
                }
                let object = &self.state.objects[id as usize];
                Some(CounterPlacement {
                    object: self.object_ref(id),
                    placed_by: event.source.controller,
                    card: object.card,
                    owner: object.owner,
                    controller: object.controller,
                    subtypes: self.granted_subtypes(id),
                    kind: self.current_kinds(id),
                    amount: Counters::of(
                        CounterKind::MinusOne,
                        u64::try_from(event.amount).capacity("wither counter quantity exhausted"),
                    ),
                    // CR120.2 and 614.16: combat is a turn-based action;
                    // noncombat damage comes from an effect of a spell or ability.
                    effect: !event.combat,
                    entry_index: None,
                    private: false,
                })
            })
            .collect()
    }
    /// The choosing player places blight counters, even when an opponent's
    /// resolving ability instructed them to do so. Costs are not effects.
    pub(super) fn blight_counters(
        &mut self,
        choices: Vec<(Player, ObjectRef)>,
        amount: u32,
        effect: bool,
        mode: ResumeMode,
    ) {
        let placements = choices
            .into_iter()
            .filter_map(|(player, object)| {
                if !self.on_battlefield(object.card) || self.object_ref(object.card) != object {
                    return None;
                }
                let o = &self.state.objects[object.card as usize];
                Some(CounterPlacement {
                    placed_by: player,
                    object,
                    card: o.card,
                    owner: o.owner,
                    controller: o.controller,
                    subtypes: self.granted_subtypes(object.card),
                    kind: self.current_kinds(object.card),
                    amount: Counters::of(CounterKind::MinusOne, u64::from(amount)),
                    effect,
                    entry_index: None,
                    private: false,
                })
            })
            .collect();
        self.stage_counter_placements(placements, mode);
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
            if !self.can_receive_counters(id) {
                return;
            }
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
                    ReplacementSpec::FirstTokensCopyAttached => {
                        b.first_this_turn && self.crown_recipe(source).is_some()
                    }
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
    fn crown_recipe(&self, source: ObjectRef) -> Option<CopyRecipe> {
        if !self.on_battlefield(source.card) || self.object_ref(source.card) != source {
            return None;
        }
        let target = self.state.objects[source.card as usize].attached_to?;
        (self.on_battlefield(target.card)
            && self.object_ref(target.card) == target
            && self.is_creature(target.card))
        .then(|| self.copiable_recipe(target))
    }
    fn begin_token_event(&mut self, player: Player, amount: u128) -> bool {
        let history = &mut self.state.players[player.index()].token_creation_attempted_this_turn;
        let first = !*history;
        if amount > 0 {
            *history = true;
        }
        first
    }
    fn modify_tokens(&self, choice: ReplacementEffectView, b: &mut TokenBatch) {
        match choice.source_card.replacement_effects()[choice.ability as usize] {
            ReplacementSpec::FirstTokensCopyAttached => {
                let recipe = self
                    .crown_recipe(choice.source)
                    .expect("applicable exact equipped creature");
                b.recipes = b
                    .groups
                    .iter_mut()
                    .map(|g| {
                        g.token = recipe.definition;
                        Some(recipe)
                    })
                    .collect();
            }
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
                for (index, g) in b.groups.iter_mut().enumerate() {
                    if g.token.is_artifact() {
                        g.token = token;
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
        self.stage_copy_creation_completion(player, recipe, amount, TokenCompletion::None);
    }
    pub(super) fn stage_attacking_copy_creation(&mut self, player: Player, recipe: CopyRecipe) {
        self.stage_copy_creation_completion(player, recipe, 1, TokenCompletion::AttackingCopies);
    }
    fn stage_copy_creation_completion(
        &mut self,
        player: Player,
        recipe: CopyRecipe,
        amount: u128,
        completion: TokenCompletion,
    ) {
        if amount == 0 {
            return;
        }
        let attacking = matches!(completion, TokenCompletion::AttackingCopies);
        let first_this_turn = self.begin_token_event(player, amount);
        let batch = TokenBatch {
            first_this_turn,
            recipes: vec![Some(recipe)],
            player,
            groups: vec![TokenGroupView {
                token: recipe.definition,
                amount,
                tapped: attacking,
            }],
            effect: true,
            used: Vec::new(),
            completion,
        };
        if !attacking
            && self.state.replacements.is_none()
            && self.token_replacements(&batch).is_empty()
        {
            self.create_copy_groups(player, &batch.groups, &batch.recipes);
            return;
        }
        let state = self.state.replacements.get_or_insert_with(|| {
            Box::new(ReplacementState {
                work: VecDeque::new(),
                active: None,
                attachment: None,
                optional_token: None,
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
        let first_this_turn = self.begin_token_event(player, recipients.len() as u128);
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
                optional_token: None,
                resume: None,
                mode: ResumeMode::Program,
            })
        });
        state.work.push_back(Work::Tokens(TokenBatch {
            first_this_turn,
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
        let first_this_turn = self.begin_token_event(player, amount);
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
                                | ReplacementSpec::FirstTokensCopyAttached
                        ) || matches!(s, ReplacementSpec::ReplaceArtifactTokens(_))
                            && token.is_artifact()
                    })
            })
        {
            self.create_token_batch_capture(player, token, amount, tapped, None, false);
            return;
        }

        let batch = TokenBatch {
            first_this_turn,
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
                optional_token: None,
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
                optional_token: None,
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
                    Some(Work::DamageResults(events)) => {
                        self.commit_unprevented_damage_batch(&events);
                        continue;
                    }
                    Some(Work::Counters(batch)) => state.active = Some(Event::Counters(batch)),
                    Some(Work::Tokens(batch)) => state.active = Some(Event::Tokens(batch)),
                    Some(Work::CompleteTokens {
                        player,
                        completion,
                        created,
                    }) => {
                        self.complete_tokens(&mut state, player, completion, created);
                        continue;
                    }
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
                                let first_this_turn =
                                    self.begin_token_event(player, u128::from(amount));
                                state.work.push_front(Work::Tokens(TokenBatch {
                                    first_this_turn,
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
                        // Damage prevention is settled before damage becomes
                        // counters; preserve those events across counter choices.
                        let events: Vec<_> = batch
                            .events
                            .into_iter()
                            .filter(|event| !self.damage_event_prevented(event))
                            .collect();
                        let placements = self.wither_damage_counters(&events);
                        if placements.is_empty() {
                            self.commit_unprevented_damage_batch(&events);
                        } else {
                            state.work.push_front(Work::DamageResults(events));
                            state.work.push_front(Work::Counters(CounterBatch {
                                placements,
                                next: 0,
                                used: Vec::new(),
                            }));
                        }
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
                            if matches!(
                                batch.completion,
                                TokenCompletion::Mobilize | TokenCompletion::AttackingCopies
                            ) {
                                let delayed_sacrifice =
                                    matches!(batch.completion, TokenCompletion::Mobilize);
                                self.state.replacements = Some(state);
                                self.start_mobilize_groups(
                                    batch.groups,
                                    batch.recipes,
                                    delayed_sacrifice,
                                );
                                return self.resume_token_entries(resume_program);
                            }
                            if !batch.recipes.is_empty() {
                                let recipients = match &batch.completion {
                                    TokenCompletion::RoleAttach(r) => vec![*r; batch.groups.len()],
                                    TokenCompletion::RoleReflexive(c) => {
                                        vec![c.recipient; batch.groups.len()]
                                    }
                                    TokenCompletion::RoleAttachMany(r) => r.clone(),
                                    _ => Vec::new(),
                                };
                                let created = self.create_copy_groups_bound(
                                    batch.player,
                                    &batch.groups,
                                    &batch.recipes,
                                    None,
                                    &recipients,
                                );
                                state.work.push_front(Work::CompleteTokens {
                                    player: batch.player,
                                    completion: batch.completion,
                                    created,
                                });
                                if let Some(entry) = self.state.entry_batch.as_mut() {
                                    assert!(entry.replacement_resume.is_none());
                                    entry.replacement_resume = Some(state);
                                    return self.advance_entries();
                                }
                            } else {
                                let capture = matches!(
                                    batch.completion,
                                    TokenCompletion::JobAttach(_)
                                        | TokenCompletion::Keywords { .. }
                                        | TokenCompletion::Reflexive { .. }
                                        | TokenCompletion::RoleReflexive(_)
                                );
                                let created = match &batch.completion {
                                    TokenCompletion::RoleAttach(r) => self
                                        .create_attached_role_groups(
                                            batch.player,
                                            &batch.groups,
                                            *r,
                                        ),
                                    TokenCompletion::RoleReflexive(c) => self
                                        .create_token_groups_attached(
                                            batch.player,
                                            &batch.groups,
                                            None,
                                            true,
                                            Some(c.recipient),
                                        ),
                                    TokenCompletion::RoleAttachMany(r) => self
                                        .create_token_groups_per_recipient(
                                            batch.player,
                                            &batch.groups,
                                            r,
                                        ),
                                    _ => self.create_token_groups(
                                        batch.player,
                                        &batch.groups,
                                        None,
                                        capture,
                                    ),
                                };
                                self.complete_tokens(
                                    &mut state,
                                    batch.player,
                                    batch.completion,
                                    created,
                                );
                            }
                        }
                        [only]
                            if matches!(
                                only.source_card.replacement_effects()[only.ability as usize],
                                ReplacementSpec::FirstTokensCopyAttached
                            ) =>
                        {
                            state.optional_token = Some(*only);
                            self.state.pending = Pending::Replacement {
                                player: batch.player,
                            };
                            self.state.replacements = Some(state);
                            return true;
                        }
                        [only] => self.modify_tokens(*only, batch),
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
    pub(super) fn choose_token_copy_optional(&mut self, accept: bool) {
        let mut state = self.state.replacements.take().unwrap();
        let choice = state.optional_token.take().unwrap();
        let Some(Event::Tokens(batch)) = state.active.as_mut() else {
            unreachable!()
        };
        if accept {
            self.modify_tokens(choice, batch);
        } else {
            batch.used.push((choice.source, choice.ability));
        }
        self.state.replacements = Some(state);
        self.advance_replacements(true);
    }
    pub(super) fn defer_token_completion(
        &mut self,
        player: Player,
        completion: TokenCompletion,
        created: Vec<ObjectRef>,
    ) {
        self.state
            .replacements
            .as_mut()
            .expect("owned token replacement continuation")
            .work
            .push_front(Work::CompleteTokens {
                player,
                completion,
                created,
            });
    }
    pub(super) fn resume_token_entries(&mut self, resume_program: bool) -> bool {
        if let Some(entry) = self.state.entry_batch.as_mut() {
            assert!(entry.replacement_resume.is_none());
            entry.replacement_resume = self.state.replacements.take();
            self.advance_entries()
        } else if self.state.mobilize.is_some() {
            true
        } else {
            self.advance_replacements(resume_program)
        }
    }
    fn complete_tokens(
        &mut self,
        state: &mut ReplacementState,
        player: Player,
        completion: TokenCompletion,
        mut created: Vec<ObjectRef>,
    ) {
        created.retain(|r| self.on_battlefield(r.card) && self.object_ref(r.card) == *r);
        match completion {
            TokenCompletion::Keywords { source, keywords } => {
                let ids = created.iter().map(|r| r.card).collect::<Vec<_>>();
                self.grant_keyword_effects(&ids, source, keywords);
            }
            TokenCompletion::Reflexive { item, ability } if !created.is_empty() => {
                self.queue_reflexive(&item, ability, None)
            }
            TokenCompletion::RoleReflexive(c)
                if created.iter().any(|r| {
                    self.object_view(r.card).card == c.token
                        && self.object_view(r.card).attached_to == Some(c.recipient)
                }) =>
            {
                self.queue_reflexive(&c.item, c.ability, Some(c.recipient))
            }
            TokenCompletion::JobAttach(source) => {
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
                            self.state.pending = Pending::CreatedToken { player };
                        }
                    }
                }
            }
            TokenCompletion::MobilizeDelayed(item) => self.record_mobilize_tokens(&item, created),
            _ => {}
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
                if matches!(
                    choice.source_card.replacement_effects()[ability as usize],
                    ReplacementSpec::FirstTokensCopyAttached
                ) {
                    state.optional_token = Some(choice);
                    self.state.replacements = Some(state);
                    return;
                }
                self.modify_tokens(choice, batch);
            }
        }
        self.state.replacements = Some(state);
        self.advance_replacements(true);
    }
    pub(super) fn visit_replacement_actions(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        if self
            .state
            .replacements
            .as_ref()
            .unwrap()
            .optional_token
            .is_some()
        {
            return emit(Action::ChooseOptional { accept: true })
                || emit(Action::ChooseOptional { accept: false });
        }
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
                kind: if self.state.replacements.as_ref()?.optional_token.is_some() {
                    "TokensOptional"
                } else {
                    "Tokens"
                },
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
                effects: self
                    .state
                    .replacements
                    .as_ref()?
                    .optional_token
                    .map_or_else(|| self.token_replacements(batch), |choice| vec![choice]),
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
            optional_token: None,
            resume: None,
            mode: ResumeMode::Program,
        };
        let empty = frame.bytes();
        frame.active = Some(Event::Tokens(TokenBatch {
            first_this_turn: true,
            recipes: Vec::new(),
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
            prepared: true,
            discard_context: None,
            moves: Vec::new(),
            choices: Vec::new(),
            next: 0,
            counter_next: 0,
            entry_bonuses_ready: false,
            counter_order: Vec::new(),
            timestamp_order: Vec::new(),
            timestamps_ready: false,
            resume: None,
            shuffle_after: None,
            replacement_resume: None,
            tap_events: Vec::new(),
            tap_next: 0,
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
            optional_token: None,
            resume: None,
            mode: ResumeMode::Program,
        };
        state.active = Some(Event::Tokens(TokenBatch {
            first_this_turn: true,
            recipes: Vec::new(),
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
