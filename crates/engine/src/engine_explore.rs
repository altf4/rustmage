//! Public explore transactions, exact incarnations and bounded owned knowledge.
use super::*;
use crate::cards::StaticAbilityFragment;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct RevealedLibraryCard {
    pub object: ObjectRef,
    pub card: Card,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ExplorerView {
    pub kinds: u16,
    pub object: ObjectRef,
    pub card: Card,
    pub controller: Player,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ExplorerView {
    pub(super) fn from_view(view: &ObjectView) -> Self {
        Self {
            kinds: view.kinds,
            object: view.reference,
            card: view.card,
            controller: view.controller,
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ExploreChoiceView {
    pub explorer: ExplorerView,
    pub revealed: RevealedLibraryCard,
    pub counter_added: bool,
}
/// Latest historical completed event; its references never claim current positions.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ExploreEventView {
    pub sequence: u64,
    pub explorer: ExplorerView,
    pub revealed: Option<RevealedLibraryCard>,
    pub destination: Option<Zone>,
    pub result: Option<ObjectRef>,
    pub counter_added: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Explorer {
    AwaitingEntry(ObjectRef),
    Ready(ExplorerView),
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ExploreSequence {
    explorer: Explorer,
    remaining: u128,
    /// Frozen number of equivalent scry replacements left for this event.
    scries: Option<usize>,
    pub(super) choice: Option<ExploreChoiceView>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ExploreState {
    pub sources: Vec<CardId>,
    pub known_top: [Option<ObjectRef>; 2],
    pub last: Option<ExploreEventView>,
    pub(super) sequence: u64,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ExploreState {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.sources.len() * std::mem::size_of::<CardId>()
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    fn explore_data(&mut self) -> &mut ExploreState {
        self.state
            .explore_data
            .get_or_insert_with(|| Box::new(ExploreState::default()))
    }
    pub(super) fn public_library_top(&self, p: Player) -> Option<ObjectRef> {
        self.state
            .explore_data
            .as_ref()
            .and_then(|d| d.known_top[p.index()])
    }
    pub(super) fn forget_library_top(&mut self, p: Player) {
        self.library_top_changed(p);
        if let Some(d) = &mut self.state.explore_data {
            d.known_top[p.index()] = None;
        }
    }
    pub(super) fn remember_library_top(&mut self, p: Player, object: ObjectRef) {
        assert_eq!(self.top_library_card(p), Some(object.card));
        assert_eq!(self.object_ref(object.card), object);
        self.explore_data().known_top[p.index()] = Some(object);
    }
    pub(super) fn register_explore_source(&mut self, id: CardId) {
        let d = self.explore_data();
        assert!(!d.sources.contains(&id));
        d.sources.push(id);
    }
    pub(super) fn explore_completed(
        &mut self,
        explorer: ExplorerView,
        revealed: Option<RevealedLibraryCard>,
        destination: Option<Zone>,
        result: Option<ObjectRef>,
        counter_added: bool,
    ) {
        let data = self.explore_data();
        data.sequence = data
            .sequence
            .checked_add(1)
            .capacity("explore event sequence exhausted");
        data.last = Some(ExploreEventView {
            sequence: data.sequence,
            explorer,
            revealed,
            destination,
            result,
            counter_added,
        });
        let sources = data.sources.clone();
        if explorer.kinds & 1 != 0 {
            for id in sources {
                let source = self.object_view(id);
                if source.controller == explorer.controller {
                    for (index, ability) in source.card.triggers().iter().enumerate() {
                        let matches_event = match ability.condition.event() {
                            TriggerCondition::CreatureExploresYou => true,
                            TriggerCondition::CreatureExploresCardYou { land } => {
                                revealed.is_some_and(|card| card.card.is_land() == land)
                            }
                            _ => false,
                        };
                        if matches_event && self.queue_trigger(&source, index) {
                            let packet = self.state.pending_triggers.last_mut().unwrap();
                            packet.refer_objects([explorer.object]);
                            if matches!(
                                ability.condition.event(),
                                TriggerCondition::CreatureExploresCardYou { .. }
                            ) {
                                packet.refer_objects(revealed.map(|card| card.object));
                            }
                        }
                    }
                }
            }
        }
    }
    fn live_explorer(&self, object: ObjectRef) -> Option<ExplorerView> {
        let id = object.card;
        (self.on_battlefield(id) && self.object_ref(id) == object).then(|| {
            let current = &self.state.objects[id as usize];
            ExplorerView {
                kinds: self.current_kinds(id),
                object,
                card: current.card,
                controller: current.controller,
            }
        })
    }
    fn set_explore_sequence(&mut self, explorer: Explorer, amount: u128) {
        if amount != 0 {
            let resolution = self.state.resolving.as_mut().unwrap();
            assert!(resolution.explore.is_none());
            resolution.explore = Some(Box::new(ExploreSequence {
                explorer,
                remaining: amount,
                scries: None,
                choice: None,
            }));
        }
    }
    pub(super) fn start_explore(&mut self, item: &SpellView, subject: Subject, amount: u128) {
        if amount == 0 {
            return;
        }
        let explorer = match subject {
            Subject::Source => item.source.map(|source| {
                self.live_explorer(source.object)
                    .or_else(|| {
                        item.source_quantity_lki
                            .as_ref()
                            .and_then(|facts| facts.explorer)
                    })
                    .unwrap_or(ExplorerView {
                        kinds: source.facts.kinds,
                        object: source.object,
                        card: source.card,
                        controller: source.facts.controller,
                    })
            }),
            Subject::Targets => item
                .targets
                .first()
                .and_then(|bound| bound.object)
                .and_then(|object| self.live_explorer(object)),
            _ => unreachable!("unsupported explore subject"),
        };
        if let Some(explorer) = explorer {
            self.set_explore_sequence(Explorer::Ready(explorer), amount);
        }
    }
    pub(super) fn return_target_then_explore(&mut self, item: &SpellView, amount: u128) {
        let Some(object) = item.targets.first().and_then(|bound| bound.object) else {
            return;
        };
        if self.object_ref(object.card) != object
            || self.state.objects[object.card as usize].zone != Zone::Graveyard
        {
            return;
        }
        let mut movement = self.entry_move(object.card, Zone::Battlefield, false);
        movement.controller = item.controller;
        self.set_explore_sequence(Explorer::AwaitingEntry(object), amount);
        self.stage_entries(vec![movement], None);
    }
    pub(super) fn capture_returned_explorer(&mut self, events: &[ZoneChange]) {
        let Some(Explorer::AwaitingEntry(origin)) = self
            .state
            .resolving
            .as_ref()
            .and_then(|resolution| resolution.explore.as_ref())
            .map(|sequence| sequence.explorer)
        else {
            return;
        };
        let explorer = events
            .iter()
            .find(|event| {
                event.moved
                    && event.before.reference == origin
                    && event.destination == Zone::Battlefield
            })
            .and_then(|event| self.live_explorer(event.after));
        if let Some(explorer) = explorer {
            self.state
                .resolving
                .as_mut()
                .unwrap()
                .explore
                .as_mut()
                .unwrap()
                .explorer = Explorer::Ready(explorer);
        }
    }
    fn pre_explore_scries(&self, explorer: ExplorerView) -> usize {
        if explorer.kinds & 1 == 0 {
            return 0;
        }
        self.state
            .static_sources
            .iter()
            .filter(|&&id| {
                self.state.objects[id as usize].controller == explorer.controller
                    && self.active_rule_fragment(id, |fragment| {
                        matches!(
                            fragment,
                            StaticAbilityFragment::ScryBeforeControlledCreatureExplores
                        )
                    })
            })
            .count()
    }
    /// Advance one scry or explore. The program settles replacements first and
    /// calls back before advancing to the next printed instruction.
    pub(super) fn advance_explore(&mut self) -> bool {
        let resolution = self.state.resolving.as_ref().unwrap();
        let sequence = resolution.explore.as_ref().unwrap();
        if resolution.library.is_some() || sequence.choice.is_some() {
            return true;
        }
        let Explorer::Ready(explorer) = sequence.explorer else {
            self.state.resolving.as_mut().unwrap().explore = None;
            return false;
        };
        if sequence.remaining == 0 {
            self.state.resolving.as_mut().unwrap().explore = None;
            return false;
        }
        let (explorer, scries) = sequence.scries.map_or_else(
            || {
                let current = self.live_explorer(explorer.object).unwrap_or(explorer);
                (current, self.pre_explore_scries(current))
            },
            |left| (explorer, left),
        );
        let sequence = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .explore
            .as_mut()
            .unwrap();
        sequence.explorer = Explorer::Ready(explorer);
        if scries != 0 {
            sequence.scries = Some(scries - 1);
            return self.start_library_arrangement(explorer.controller, 1, false);
        }
        sequence.scries = None;
        sequence.remaining -= 1;
        self.explore_once(explorer)
    }
    fn explore_once(&mut self, explorer: ExplorerView) -> bool {
        let player = explorer.controller;
        let revealed = self.top_library_card(player).map(|id| RevealedLibraryCard {
            object: self.object_ref(id),
            card: self.state.objects[id as usize].card,
        });
        if let Some(card) = revealed {
            self.remember_library_top(player, card.object);
            if card.card.is_land() {
                let movement = self.move_object(card.object.card, Zone::Hand);
                self.explore_completed(
                    explorer,
                    Some(card),
                    Some(movement.destination),
                    Some(movement.after),
                    false,
                );
                return false;
            }
        }
        // CR701.44a: the counter precedes the optional revealed-card choice.
        // The old source may still explore under LKI; a new incarnation is not it.
        let counter_added = self.on_battlefield(explorer.object.card)
            && self.object_ref(explorer.object.card) == explorer.object
            && !replacements::counters_prohibited(self, explorer.object);
        if counter_added {
            self.effect_counter_by(
                explorer.object.card,
                CounterKind::PlusOne,
                1,
                explorer.controller,
                if revealed.is_some() {
                    replacements::ResumeMode::Pending
                } else {
                    replacements::ResumeMode::Program
                },
            );
        }
        if let Some(revealed) = revealed {
            self.state
                .resolving
                .as_mut()
                .unwrap()
                .explore
                .as_mut()
                .unwrap()
                .choice = Some(ExploreChoiceView {
                explorer,
                revealed,
                counter_added,
            });
            self.state.pending = Pending::Explore { player };
            true
        } else {
            if self.state.replacements.is_some() {
                self.defer_empty_explore(explorer, counter_added);
                true
            } else {
                self.explore_completed(explorer, None, None, None, counter_added);
                false
            }
        }
    }
    pub(super) fn finish_explore(&mut self, graveyard: bool) {
        let choice = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .explore
            .as_mut()
            .unwrap()
            .choice
            .take()
            .unwrap();
        let (destination, result) = if graveyard {
            let movement = self.move_object(choice.revealed.object.card, Zone::Graveyard);
            (movement.destination, movement.after)
        } else {
            (Zone::Library, choice.revealed.object)
        };
        self.explore_completed(
            choice.explorer,
            Some(choice.revealed),
            Some(destination),
            Some(result),
            choice.counter_added,
        );
        self.run_program();
    }
}
