//! Public explore transactions, exact incarnations and bounded owned knowledge.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct RevealedLibraryCard {
    pub object: ObjectRef,
    pub card: Card,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ExplorerView {
    pub kinds: u16,
    pub object: ObjectRef,
    pub card: Card,
    pub controller: Player,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ExploreChoiceView {
    pub explorer: ExplorerView,
    pub revealed: RevealedLibraryCard,
    pub counter_added: bool,
}
/// Latest historical completed event; its references never claim current positions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ExploreEventView {
    pub sequence: u64,
    pub explorer: ExplorerView,
    pub revealed: Option<RevealedLibraryCard>,
    pub destination: Option<Zone>,
    pub result: Option<ObjectRef>,
    pub counter_added: bool,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ExploreState {
    pub sources: Vec<CardId>,
    pub known_top: [Option<ObjectRef>; 2],
    pub last: Option<ExploreEventView>,
    pub(super) sequence: u64,
}
impl ExploreState {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.sources.len() * std::mem::size_of::<CardId>()
    }
}
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
        if let Some(d) = &mut self.state.explore_data {
            d.known_top[p.index()] = None;
        }
    }
    pub(super) fn remember_library_top(&mut self, p: Player, object: ObjectRef) {
        assert_eq!(
            self.state.players[p.index()].library.last(),
            Some(&object.card)
        );
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
                if source.owner == explorer.controller {
                    for (index, ability) in source.card.triggers().iter().enumerate() {
                        if ability.condition.event() == TriggerCondition::CreatureExploresYou {
                            self.queue_trigger(&source, index);
                        }
                    }
                }
            }
        }
    }
    pub(super) fn start_explore(&mut self, item: &SpellView, subject: Subject) -> bool {
        let explorer = match subject {
            Subject::Source => ExplorerView {
                kinds: if self.on_battlefield(item.card)
                    && self.object_ref(item.card) == item.object
                {
                    self.current_kinds(item.card)
                } else {
                    item.source_facts.kinds
                },
                object: item.object,
                card: item.source_card,
                controller: item.controller,
            },
            Subject::Targets => {
                let Some(bound) = item.targets.first().copied() else {
                    return false;
                };
                let Target::Card(id) = bound.target else {
                    return false;
                };
                if !self.same_target_object(bound) || !self.on_battlefield(id) {
                    return false;
                }
                ExplorerView {
                    kinds: self.current_kinds(id),
                    object: bound.object.unwrap(),
                    card: self.state.objects[id as usize].card,
                    controller: self.state.objects[id as usize].owner,
                }
            }
            _ => unreachable!("unsupported explore subject"),
        };
        let player = explorer.controller;
        let revealed = self.state.players[player.index()]
            .library
            .last()
            .copied()
            .map(|id| RevealedLibraryCard {
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
            && self.object_ref(explorer.object.card) == explorer.object;
        if counter_added {
            self.effect_counter(
                explorer.object.card,
                CounterKind::PlusOne,
                1,
                if revealed.is_some() {
                    replacements::ResumeMode::Pending
                } else {
                    replacements::ResumeMode::Program
                },
            );
        }
        if let Some(revealed) = revealed {
            self.state.resolving.as_mut().unwrap().explore = Some(Box::new(ExploreChoiceView {
                explorer,
                revealed,
                counter_added,
            }));
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
        let choice = *self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .explore
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
