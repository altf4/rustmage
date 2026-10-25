//! Exact target-owner destinations suspended inside the owned resolution.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TargetLibraryChoice {
    pub source: ObjectRef,
    pub controller: Player,
    pub second_from_top: bool,
    pub targets: Vec<BoundTarget>,
}
impl TargetLibraryChoice {
    pub fn destinations(&self) -> [LibraryDestination; 2] {
        [
            if self.second_from_top {
                LibraryDestination::SecondFromTop
            } else {
                LibraryDestination::Top
            },
            LibraryDestination::Bottom,
        ]
    }
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.targets.len() * std::mem::size_of::<BoundTarget>()
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct TargetLibraryView {
    pub source: ObjectRef,
    pub target: ObjectRef,
    pub player: Player,
}
/// The latest public targeted movement, not a claim about current library position.
/// Private looks/reorders and draws never update this historical payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct TargetLibraryMovement {
    pub source: ObjectRef,
    pub card: Card,
    pub target_before: ObjectRef,
    pub target_after: ObjectRef,
    pub owner: Player,
    pub destination: LibraryDestination,
}
impl Game {
    pub(super) fn valid_library_target(&self, item: &SpellView, bound: BoundTarget) -> bool {
        matches!(bound.target, Target::Card(_))
            && self.same_target_object(bound)
            && self
                .target_spec(item)
                .group(bound.group)
                .is_some_and(|spec| self.item_target_matches(item, spec.rule, bound.target))
    }
    pub(super) fn move_target_to_library(
        &mut self,
        source: ObjectRef,
        object: ObjectRef,
        destination: LibraryDestination,
    ) {
        assert!(matches!(
            destination,
            LibraryDestination::Top
                | LibraryDestination::Bottom
                | LibraryDestination::SecondFromTop
        ));
        let owner = self.state.objects[object.card as usize].owner;
        let card = self.state.objects[object.card as usize].card;
        let token = self.state.objects[object.card as usize].is_token();
        let prior_top = self.public_library_top(owner);
        let old_len = self.state.players[owner.index()].library.len();
        let event = self.move_object(object.card, Zone::Library);
        if event.moved && event.destination == Zone::Library {
            let library = &mut self.state.players[owner.index()].library;
            if !token
                && matches!(
                    destination,
                    LibraryDestination::Bottom | LibraryDestination::SecondFromTop
                )
            {
                assert_eq!(library.pop(), Some(object.card));
                let index = if destination == LibraryDestination::SecondFromTop {
                    old_len.saturating_sub(1)
                } else {
                    0
                };
                library.insert(index, object.card);
            }
            if !token
                && (destination == LibraryDestination::Top
                    || (destination == LibraryDestination::SecondFromTop && old_len == 0))
            {
                self.remember_library_top(owner, event.after);
            } else if let Some(prior) = prior_top
                && self.state.players[owner.index()].library.last() == Some(&prior.card)
                && self.object_ref(prior.card) == prior
            {
                self.remember_library_top(owner, prior);
            }
            // Publish only this targeted public movement. This is a historical
            // record; later private looks, rearrangements and draws leave it alone.
            self.state.last_target_library_move = Some(Box::new(TargetLibraryMovement {
                source,
                card,
                target_before: object,
                target_after: event.after,
                owner,
                destination,
            }));
        }
    }
    pub(super) fn start_target_library(&mut self, item: &SpellView, second_from_top: bool) -> bool {
        let targets = item
            .targets
            .iter()
            .copied()
            .filter(|&bound| self.valid_library_target(item, bound))
            .collect::<Vec<_>>();
        if targets.is_empty() {
            return false;
        }
        self.state.resolving.as_mut().unwrap().target_library =
            Some(Box::new(TargetLibraryChoice {
                source: item.target_source(),
                controller: item.controller,
                second_from_top,
                targets,
            }));
        self.advance_target_library()
    }
    fn advance_target_library(&mut self) -> bool {
        loop {
            let r = self.state.resolving.as_ref().unwrap();
            let f = r.target_library.as_ref().unwrap();
            let Some(&bound) = f.targets.first() else {
                self.state.resolving.as_mut().unwrap().target_library = None;
                return false;
            };
            if self.valid_library_target(&r.item, bound) {
                self.state.pending = Pending::TargetLibrary {
                    player: self.state.objects[bound.object.unwrap().card as usize].owner,
                };
                return true;
            }
            self.state
                .resolving
                .as_mut()
                .unwrap()
                .target_library
                .as_mut()
                .unwrap()
                .targets
                .remove(0);
        }
    }
    pub(super) fn choose_library_destination(
        &mut self,
        object: ObjectRef,
        destination: LibraryDestination,
    ) {
        let r = self.state.resolving.as_ref().unwrap();
        let bound = r.target_library.as_ref().unwrap().targets[0];
        assert_eq!(bound.object, Some(object));
        if self.valid_library_target(&r.item, bound) {
            self.move_target_to_library(
                r.target_library.as_ref().unwrap().source,
                object,
                destination,
            );
        }
        self.state
            .resolving
            .as_mut()
            .unwrap()
            .target_library
            .as_mut()
            .unwrap()
            .targets
            .remove(0);
        if !self.advance_target_library() {
            self.run_program();
        }
    }
    pub(super) fn target_library_view(&self, viewer: Player) -> Option<TargetLibraryView> {
        let viewer = self.choice_information_player(viewer);
        if self.semantic_actor() != Some(viewer) {
            return None;
        }
        let f = self.state.resolving.as_ref()?.target_library.as_ref()?;
        let target = f.targets.first()?.object?;
        if self.state.objects[target.card as usize].owner != viewer {
            return None;
        }
        Some(TargetLibraryView {
            source: f.source,
            target,
            player: viewer,
        })
    }
}
