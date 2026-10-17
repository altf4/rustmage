//! Sparse live source clauses. Admission waits for complete compiler/timestamp support.
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
        let timestamp = self.next_role_timestamp();
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
        if !self
            .state
            .roles
            .as_ref()
            .is_some_and(|r| r.live_static.iter().any(|e| e.source == source))
        {
            return;
        }
        let timestamp = self.next_role_timestamp();
        for e in &mut self.state.roles.as_mut().unwrap().live_static {
            if e.source == source {
                e.timestamp = timestamp;
            }
        }
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
            let source = named(&mut g, p, "Bear Cub");
            let own = named(&mut g, p, "Bear Cub");
            let enemy = named(&mut g, p.other(), "Bear Cub");
            g.register_live_static_effects(source, &[OPPONENT]);
            let stamp = g.observe(p).live_static_sources[0].timestamp;
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
            g.state.objects[detective as usize].card = Card::from_name("Detective Token").unwrap();
            assert!(g.live_static_forbids_block(target, detective));
            g.state.objects[target as usize].card = Card::from_name("Detective Token").unwrap();
            assert_eq!(g.stats(target).power, 5); // 2 printed +2 conditional +1 Pick.
            assert!(!g.live_static_forbids_block(target, detective));
            g.state.objects[aura as usize].controller = p.other();
            assert_eq!(g.stats(target).power, 2);
            assert!(g.live_static_forbids_block(target, detective));
        }
    }
}
