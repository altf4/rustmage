//! Sparse live source clauses use current earlier-layer facts and a shared timestamp clock.
use super::*;
use crate::cards::{LiveStatRecipients, LiveStatValue, LiveStaticEffect};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LiveStaticSourceView {
    pub source: ObjectRef,
    pub timestamp: u64,
    /// Immutable catalogue data; independent of every game/parent allocation.
    pub effects: &'static [LiveStaticEffect],
}

impl Game {
    pub(super) fn register_live_static_source(&mut self, id: CardId) {
        let effects = self.state.objects[id as usize].card.live_static_effects();
        self.register_live_static_effects(id, effects);
    }
    fn register_live_static_effects(&mut self, id: CardId, effects: &'static [LiveStaticEffect]) {
        if effects.is_empty() {
            return;
        }
        let source = self.object_ref(id);
        if self
            .state
            .roles
            .as_ref()
            .is_some_and(|r| r.live_static.iter().any(|e| e.source == source))
        {
            return;
        }
        let timestamp = self.static_source_timestamp(source);
        self.state
            .roles
            .get_or_insert_with(Default::default)
            .live_static
            .push(LiveStaticSourceView {
                source,
                timestamp,
                effects,
            });
    }
    pub(super) fn forget_live_static_source(&mut self, source: ObjectRef) {
        if let Some(r) = &mut self.state.roles {
            r.live_static.retain(|e| e.source != source);
        }
    }
    pub(super) fn retimestamp_live_static_source(&mut self, source: ObjectRef) {
        if !self.state.roles.as_ref().is_some_and(|r| {
            r.live_static.iter().any(|e| e.source == source)
                || r.ability_timestamps.iter().any(|e| e.source == source)
        }) {
            return;
        }
        self.assign_static_source_timestamp(source);
    }
    fn live_static_applies(
        &self,
        row: &LiveStaticSourceView,
        effect: &LiveStaticEffect,
        id: CardId,
    ) -> bool {
        if !self.on_battlefield(id)
            || !self.is_creature(id)
            || !self.on_battlefield(row.source.card)
            || self.object_ref(row.source.card) != row.source
        {
            return false;
        }
        let source = &self.state.objects[row.source.card as usize];
        let target = &self.state.objects[id as usize];
        let reference = self.object_ref(id);
        let member = match effect.recipients {
            LiveStatRecipients::Attached => source.attached_to == Some(reference),
            LiveStatRecipients::Creatures {
                relation,
                other,
                enchanted_by_controller,
            } => {
                (match relation {
                    0 => target.controller == source.controller,
                    1 => target.controller != source.controller,
                    _ => true,
                }) && (!other || reference != row.source)
                    && (!enchanted_by_controller
                        || self.state.attachment_sources.iter().any(|&a| {
                            let aura = &self.state.objects[a as usize];
                            aura.zone == Zone::Battlefield
                                && aura.card.aura().is_some()
                                && aura.controller == source.controller
                                && aura.attached_to == Some(reference)
                        }))
            }
        };
        member
            && effect.condition.is_none_or(|c| {
                let matching = target.controller == source.controller
                    && (target.card.has_subtype(c.subtype)
                        || self.granted_subtypes(id).contains(c.subtype));
                matching != c.negated
            })
    }
    fn live_stat_value(&self, value: LiveStatValue, source: ObjectRef) -> i128 {
        match value {
            LiveStatValue::Fixed(value) => value,
            LiveStatValue::ControllerLife => {
                self.state.players[self.state.objects[source.card as usize].controller.index()].life
            }
        }
    }
    pub(super) fn apply_live_base_stat_layers(
        &self,
        id: CardId,
        mut stats: Stats,
        power_time: &mut Option<u64>,
        toughness_time: &mut Option<u64>,
    ) -> Stats {
        if let Some(r) = &self.state.roles {
            for row in &r.live_static {
                for effect in row.effects {
                    if !self.live_static_applies(row, effect, id) {
                        continue;
                    }
                    if let Some(power) = effect.power
                        && power_time.is_none_or(|t| row.timestamp > t)
                    {
                        stats.power = self.live_stat_value(power, row.source);
                        *power_time = Some(row.timestamp);
                    }
                    if let Some(toughness) = effect.toughness
                        && toughness_time.is_none_or(|t| row.timestamp > t)
                    {
                        stats.toughness = self.live_stat_value(toughness, row.source);
                        *toughness_time = Some(row.timestamp);
                    }
                }
            }
        }
        stats
    }
    pub(super) fn live_static_bonus(&self, id: CardId) -> Stats {
        let mut stats = Stats::default();
        if let Some(r) = &self.state.roles {
            for row in &r.live_static {
                for effect in row.effects {
                    if self.live_static_applies(row, effect, id) {
                        stats += Stats {
                            power: effect.bonus_power,
                            toughness: effect.bonus_toughness,
                        };
                    }
                }
            }
        }
        stats
    }
    pub(super) fn live_static_keywords(&self, id: CardId) -> Keywords {
        let mut keywords = Keywords::NONE;
        if let Some(r) = &self.state.roles {
            for row in &r.live_static {
                for effect in row.effects {
                    if self.live_static_applies(row, effect, id) {
                        keywords = keywords.union(effect.keywords);
                    }
                }
            }
        }
        keywords
    }
    pub(super) fn live_static_forbids_block(&self, blocker: CardId, attacker: CardId) -> bool {
        self.state.roles.as_ref().is_some_and(|r| {
            r.live_static.iter().any(|row| {
                row.effects.iter().any(|effect| {
                    self.live_static_applies(row, effect, blocker)
                        && effect.cannot_block_subtype.is_some_and(|subtype| {
                            self.state.objects[attacker as usize]
                                .card
                                .has_subtype(subtype)
                                || self.granted_subtypes(attacker).contains(subtype)
                        })
                })
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{scenario, spawn};
    use super::*;
    use crate::cards::LiveStatCondition;
    const OPPONENT: LiveStaticEffect = LiveStaticEffect {
        recipients: LiveStatRecipients::Creatures {
            relation: 1,
            other: false,
            enchanted_by_controller: false,
        },
        condition: None,
        power: None,
        toughness: Some(LiveStatValue::Fixed(1)),
        bonus_power: 0,
        bonus_toughness: 0,
        keywords: Keywords::NONE,
        cannot_block_subtype: None,
    };
    const ATTACHED_LIFE: LiveStaticEffect = LiveStaticEffect {
        recipients: LiveStatRecipients::Attached,
        condition: None,
        power: Some(LiveStatValue::ControllerLife),
        toughness: Some(LiveStatValue::ControllerLife),
        bonus_power: 0,
        bonus_toughness: 0,
        keywords: Keywords::NONE,
        cannot_block_subtype: None,
    };
    const OTHER_ENCHANTED: LiveStaticEffect = LiveStaticEffect {
        recipients: LiveStatRecipients::Creatures {
            relation: 0,
            other: true,
            enchanted_by_controller: true,
        },
        condition: None,
        power: Some(LiveStatValue::Fixed(4)),
        toughness: Some(LiveStatValue::Fixed(4)),
        bonus_power: 0,
        bonus_toughness: 0,
        keywords: Keywords::FLYING,
        cannot_block_subtype: None,
    };
    const DETECTIVE_BONUS: LiveStaticEffect = LiveStaticEffect {
        recipients: LiveStatRecipients::Attached,
        condition: Some(LiveStatCondition {
            subtype: "Detective",
            negated: false,
        }),
        power: None,
        toughness: None,
        bonus_power: 2,
        bonus_toughness: 2,
        keywords: Keywords::NONE,
        cannot_block_subtype: None,
    };
    const NONDETECTIVE: LiveStaticEffect = LiveStaticEffect {
        recipients: LiveStatRecipients::Attached,
        condition: Some(LiveStatCondition {
            subtype: "Detective",
            negated: true,
        }),
        power: Some(LiveStatValue::Fixed(1)),
        toughness: Some(LiveStatValue::Fixed(1)),
        bonus_power: 0,
        bonus_toughness: 0,
        keywords: Keywords::NONE,
        cannot_block_subtype: Some("Detective"),
    };
    fn named(g: &mut Game, p: Player, name: &str) -> CardId {
        spawn(g, p, Card::from_name(name).unwrap(), Zone::Battlefield)
    }
    #[test]
    fn live_static_primitives_track_controller_and_late_recipient_without_retimestamp() {
        for p in [Player::P0, Player::P1] {
            let mut g = scenario();
            let source = named(&mut g, p, "Doc Ock, Sinister Scientist");
            let own = named(&mut g, p, "Bear Cub");
            let enemy = named(&mut g, p.other(), "Bear Cub");
            g.register_live_static_effects(source, &[OPPONENT]);
            let stamp = g.observe(p).live_static_sources[0].timestamp;
            assert_eq!(
                stamp,
                g.observe(p)
                    .base_stat_effects
                    .iter()
                    .find(|e| e.source.card == source)
                    .unwrap()
                    .timestamp
            );
            assert_eq!(g.stats(own).toughness, 2);
            assert_eq!(g.stats(enemy).toughness, 1);
            let late = named(&mut g, p.other(), "Bear Cub");
            assert_eq!(g.stats(late).toughness, 1);
            g.state.objects[source as usize].controller = p.other();
            assert_eq!(g.stats(own).toughness, 1);
            assert_eq!(g.stats(enemy).toughness, 2);
            assert_eq!(g.observe(p).live_static_sources[0].timestamp, stamp);
            let before = g.snapshot();
            let mut child = {
                let parent = g.clone();
                parent.clone()
            };
            g.move_object(source, Zone::Hand);
            child.move_object(source, Zone::Hand);
            assert_eq!(g.snapshot(), child.snapshot());
            assert_eq!(g.stats(own).toughness, 2);
            assert!(g.observe(p).live_static_sources.is_empty());
            g.restore(&before).unwrap();
            assert_eq!(g.stats(own).toughness, 1);
        }
    }
    #[test]
    fn live_static_primitives_aura_membership_uses_controller_not_owner_or_equipment() {
        for p in [Player::P0, Player::P1] {
            let mut g = scenario();
            let source = named(&mut g, p, "Bear Cub");
            let target = named(&mut g, p, "Bear Cub");
            let aura = named(&mut g, p.other(), "Pacifism");
            g.state.objects[aura as usize].attached_to = Some(g.object_ref(target));
            g.register_live_static_effects(source, &[OTHER_ENCHANTED]);
            assert_eq!(g.stats(target).power, 2);
            assert!(!g.keywords(target).contains(Keywords::FLYING));
            g.state.objects[aura as usize].controller = p;
            assert_eq!(g.stats(target).power, 4);
            assert!(g.keywords(target).contains(Keywords::FLYING));
            g.state.objects[aura as usize].attached_to = Some(g.object_ref(source));
            assert_eq!(g.stats(source).power, 2);
            g.state.objects[aura as usize].attached_to = None;
            let equipment = named(&mut g, p, "Goldvein Pick");
            assert!(g.attach_equipment(g.object_ref(equipment), g.object_ref(target)));
            assert_eq!(g.stats(target).power, 3);
            assert!(!g.keywords(target).contains(Keywords::FLYING));
        }
    }
    #[test]
    fn live_static_primitives_signed_life_and_detective_condition_are_earlier_layer_facts() {
        for p in [Player::P0, Player::P1] {
            let mut g = scenario();
            let source = named(&mut g, p, "Goldvein Pick");
            let target = named(&mut g, p, "Bear Cub");
            assert!(g.attach_equipment(g.object_ref(source), g.object_ref(target)));
            g.register_live_static_effects(source, &[ATTACHED_LIFE]);
            let stamp = g.observe(p).live_static_sources[0].timestamp;
            g.state.players[p.index()].life = -2; // Primitive arithmetic probe, not a naturally legal ongoing game.
            assert_eq!((g.stats(target).power, g.stats(target).toughness), (-1, -1));
            g.state.players[p.index()].life = 17;
            assert_eq!((g.stats(target).power, g.stats(target).toughness), (18, 18));
            assert_eq!(g.observe(p).live_static_sources[0].timestamp, stamp);
            g.forget_live_static_source(g.object_ref(source));
            let aura = named(&mut g, p, "Pacifism");
            g.state.objects[aura as usize].attached_to = Some(g.object_ref(target));
            g.register_live_static_effects(aura, &[DETECTIVE_BONUS, NONDETECTIVE]);
            assert_eq!((g.stats(target).power, g.stats(target).toughness), (2, 2)); // 1/1 base plus Pick.
            let detective = named(&mut g, p.other(), "Person of Interest");
            assert!(!g.live_static_forbids_block(target, detective));
            g.state.objects[detective as usize].card =
                Card::from_name("April, Reporter of the Weird").unwrap();
            assert!(g.live_static_forbids_block(target, detective));
            g.state.objects[target as usize].card =
                Card::from_name("April, Reporter of the Weird").unwrap();
            assert_eq!(g.stats(target).power, 5); // 2 printed +2 conditional +1 Pick.
            assert!(!g.live_static_forbids_block(target, detective));
            g.state.objects[aura as usize].controller = p.other();
            assert_eq!(g.stats(target).power, 2);
            assert!(g.live_static_forbids_block(target, detective));
        }
    }
}

#[cfg(test)]
mod timestamp_tests {
    use super::super::tests::{scenario, spawn};
    use super::*;
    fn checked(g: &mut Game, action: Action) {
        let actor = g.actor().unwrap();
        let revision = g.revision();
        let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
        let before = g.snapshot();
        assert_eq!(
            g.apply(actor.other(), action.clone()),
            Err(Error::WrongActor)
        );
        assert_eq!(g.snapshot(), before);
        let mut child = {
            let parent = g.clone();
            parent.clone()
        };
        child.apply_index(actor, revision, index).unwrap();
        g.apply(actor, action.clone()).unwrap();
        assert_eq!(child.snapshot(), g.snapshot());
        for p in [Player::P0, Player::P1] {
            assert_eq!(
                serde_json::to_value(child.observe(p)).unwrap(),
                serde_json::to_value(g.observe(p)).unwrap()
            );
        }
        let after = g.snapshot();
        g.restore(&before).unwrap();
        assert_eq!(
            g.apply_index(actor, revision, index),
            Err(Error::StaleRevision)
        );
        g.apply(actor, action).unwrap();
        assert_eq!(g.snapshot(), after);
    }
    #[test]
    fn live_static_primitives_simultaneous_entries_choose_apnap_owned_order() {
        for active in [Player::P0, Player::P1] {
            for reverse in [false, true] {
                let mut g = scenario();
                g.state.active = active;
                g.state.priority = active;
                let mut ids = Vec::new();
                for p in [active.other(), active] {
                    for name in ["Doc Ock, Sinister Scientist", "Snowmelt Stag"] {
                        ids.push(spawn(
                            &mut g,
                            p,
                            Card::from_name(name).unwrap(),
                            Zone::Graveyard,
                        ));
                    }
                }
                let moves = ids
                    .iter()
                    .map(|&id| g.entry_move(id, Zone::Battlefield, false))
                    .collect();
                g.stage_entries(moves, None);
                assert!(g.advance_entries());
                assert_eq!(g.actor(), Some(active));
                assert!(
                    ids.iter()
                        .all(|&id| g.state.objects[id as usize].zone == Zone::Graveyard)
                );
                let view = g.observe(active.other()).entry_timestamps.unwrap();
                assert_eq!(view.controller, active);
                assert_eq!(view.candidates.len(), 2);
                let illegal = Action::ChooseEntryTimestamp {
                    object: g.object_ref(ids[0]),
                };
                let before = g.snapshot();
                assert!(g.apply(active, illegal).is_err());
                assert_eq!(g.snapshot(), before);
                let first = if reverse { 3 } else { 2 };
                let object = g.object_ref(ids[first]);
                checked(&mut g, Action::ChooseEntryTimestamp { object });
                assert_eq!(g.actor(), Some(active.other()));
                let second = if reverse { 0 } else { 1 };
                let object = g.object_ref(ids[second]);
                checked(&mut g, Action::ChooseEntryTimestamp { object });
                assert!(g.state.entry_batch.is_none());
                assert!(ids.iter().all(|&id| g.on_battlefield(id)));
                let rows = &g.state.roles.as_ref().unwrap().base_stats;
                let stamp = |id| rows.iter().find(|v| v.source.card == id).unwrap().timestamp;
                assert!(stamp(ids[first]) < stamp(ids[5 - first]));
                assert!(stamp(ids[2]) < stamp(ids[0]) && stamp(ids[3]) < stamp(ids[1]));
                assert!(stamp(ids[second]) < stamp(ids[1 - second]));
                assert!(g.observe(active).entry_timestamps.is_none());
                assert!(!g.state.roles.as_ref().unwrap().deferred_timestamps);
            }
        }
    }
    #[test]
    fn live_static_primitives_departed_tokens_cannot_reenter_or_receive_timestamp() {
        for p in [Player::P0, Player::P1] {
            let mut g = scenario();
            g.state.active = p;
            g.state.priority = p;
            let bear = spawn(
                &mut g,
                p,
                Card::from_name("Bear Cub").unwrap(),
                Zone::Battlefield,
            );
            let doc = spawn(
                &mut g,
                p,
                Card::from_name("Doc Ock, Sinister Scientist").unwrap(),
                Zone::Graveyard,
            );
            let curse = Card::catalogue()
                .find(|c| c.role() == Some(crate::cards::RoleKind::Cursed))
                .unwrap();
            let aura = spawn(&mut g, p, curse, Zone::Exile);
            let mut m = g.entry_move(aura, Zone::Battlefield, false);
            m.attachment = Some(g.object_ref(bear));
            let d = g.entry_move(doc, Zone::Battlefield, false);
            g.stage_entries(vec![m, d], None);
            assert!(g.state.entry_batch.is_none());
            assert!(g.on_battlefield(doc));
            assert_eq!(g.state.objects[aura as usize].zone, Zone::Exile);
            let r = g.state.roles.as_ref().unwrap();
            assert!(r.attached.is_empty());
            assert_eq!(r.base_stats.len(), 1);
            assert_ne!(r.base_stats[0].timestamp, 0);
            assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (2, 2));
        }
    }
}
