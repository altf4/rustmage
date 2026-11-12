//! Shared-type-line split permanents and the turn-based choices that open doors.
use super::ability_world::CharacteristicWorld;
use super::*;
use crate::cards::{RoomSpec, StaticAbilityFragment, StaticAbilityGroup};

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct RoomDoors {
    pub left: bool,
    pub right: bool,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl RoomDoors {
    pub(super) fn is_open(self, door: RoomDoor) -> bool {
        match door {
            RoomDoor::Left => self.left,
            RoomDoor::Right => self.right,
        }
    }

    fn with(self, door: RoomDoor, open: bool) -> Self {
        match door {
            RoomDoor::Left => Self { left: open, ..self },
            RoomDoor::Right => Self {
                right: open,
                ..self
            },
        }
    }

    fn complete(self) -> bool {
        self.left && self.right
    }

    pub(super) fn active_halves(self, spec: &RoomSpec) -> impl Iterator<Item = Card> {
        RoomDoor::ALL
            .into_iter()
            .filter(move |&door| self.is_open(door))
            .map(move |door| spec.halves[door.index()])
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomChange {
    Unlock,
    Lock,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomOperation {
    Unlock,
    Lock,
    Either,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl RoomOperation {
    fn permits(self, change: RoomChange) -> bool {
        self == Self::Either
            || matches!(
                (self, change),
                (Self::Unlock, RoomChange::Unlock) | (Self::Lock, RoomChange::Lock)
            )
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoomSelection {
    /// Choose one locked or unlocked door of an already targeted Room.
    Target,
    /// Choose a legal Room and door without targeting during resolution.
    Controlled,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct RoomChoiceView {
    pub player: Player,
    pub target: Option<ObjectRef>,
    pub operation: RoomOperation,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct RoomUnlockPaymentView {
    pub object: ObjectRef,
    pub door: RoomDoor,
    pub controller: Player,
    pub cost: ManaCost,
    #[serde(skip)]
    definition: Card,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl RoomUnlockPaymentView {
    pub(super) fn goal(self) -> FundingGoal {
        FundingGoal {
            actor: self.controller,
            required: self.cost.colored,
            generic: self.cost.generic,
            reserve: None,
            source: None,
        }
    }

    pub(super) fn source_is_current<W: CharacteristicWorld + ?Sized>(self, world: &W) -> bool {
        world.reference(self.object.card) == self.object
            && world.object(self.object.card).controller == self.controller
            && world.definition(self.object.card) == self.definition
            && doors(world, self.object.card).is_some_and(|doors| !doors.is_open(self.door))
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) type Rows = Option<SharedVec<(ObjectRef, RoomDoors)>>;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn owned_bytes(rows: &Rows) -> usize {
    rows.as_ref().map_or(0, |rows| {
        std::mem::size_of::<Vec<(ObjectRef, RoomDoors)>>()
            + rows.len() * std::mem::size_of::<(ObjectRef, RoomDoors)>()
    })
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn doors<W: CharacteristicWorld + ?Sized>(world: &W, id: CardId) -> Option<RoomDoors> {
    if world.object(id).zone != Zone::Battlefield || world.definition(id).room().is_none() {
        return None;
    }
    let reference = world.reference(id);
    Some(
        world
            .game()
            .state
            .room_openings
            .as_ref()
            .and_then(|rows| rows.iter().find(|(object, _)| *object == reference))
            .map_or(RoomDoors::default(), |(_, doors)| *doors),
    )
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn printed_group_available<W: CharacteristicWorld + ?Sized>(
    world: &W,
    object: ObjectRef,
    group: &StaticAbilityGroup,
) -> bool {
    let Some(spec) = world.definition(object.card).room() else {
        return true;
    };
    spec.static_groups
        .iter()
        .find(|&&(line, _)| line == group.line)
        .is_none_or(|&(_, door)| doors(world, object.card).is_some_and(|doors| doors.is_open(door)))
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn printed_trigger_available(
    source: &ObjectView,
    definition: Card,
    index: usize,
) -> bool {
    definition.room().is_none_or(|spec| {
        spec.triggers
            .iter()
            .find(|&&(slot, _)| usize::from(slot) == index)
            .is_none_or(|&(_, door)| source.room_doors.is_some_and(|doors| doors.is_open(door)))
    })
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn battlefield_name<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> Option<&'static str> {
    let room = world.definition(id).room()?;
    match doors(world, id)? {
        RoomDoors {
            left: false,
            right: false,
        } => Some(""),
        RoomDoors {
            left: true,
            right: false,
        } => Some(room.halves[RoomDoor::Left.index()].name()),
        RoomDoors {
            left: false,
            right: true,
        } => Some(room.halves[RoomDoor::Right.index()].name()),
        RoomDoors {
            left: true,
            right: true,
        } => Some(world.copiable_name(id)),
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn battlefield_colors<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> Option<u8> {
    let room = world.definition(id).room()?;
    Some(
        doors(world, id)?
            .active_halves(room)
            .fold(0, |mask, card| mask | card.colors()),
    )
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn door_names<W: CharacteristicWorld + ?Sized>(
    world: &W,
    player: Player,
) -> impl Iterator<Item = &'static str> + '_ {
    (0..world.game().state.objects.len())
        .filter_map(move |index| {
            let id = index as CardId;
            let object = world.object(id);
            if object.controller != player || object.zone != Zone::Battlefield {
                return None;
            }
            let room = world.definition(id).room()?;
            world
                .subtypes(id)
                .has(world.definition(id), "Room")
                .then(|| {
                    doors(world, id)
                        .expect("Room doors")
                        .active_halves(room)
                        .map(Card::name)
                })
        })
        .flatten()
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn visit_room_casts(
        &self,
        actor: Player,
        card: CardId,
        sorcery: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let object = &self.state.objects[card as usize];
        if !object.is_card() || !object.card.is_supported() {
            return false;
        }
        CastFace::choices(object.card).any(|face| {
            let Some(door) = face.room_door() else {
                return false;
            };
            let definition = face.definition(object.card).expect("Room half");
            let allowed = match object.zone {
                Zone::Hand => object.owner == actor,
                Zone::Exile => self.can_play_exiled(actor, card),
                Zone::Graveyard => self.has_graveyard_cast_face(actor, card, face),
                Zone::Library => self.can_cast_library_face(actor, card, definition),
                _ => false,
            };
            allowed
                && [false, true].into_iter().any(|flash_surcharge| {
                    self.permitted_cast_available(
                        actor,
                        card,
                        CastMethod::Normal,
                        face,
                        sorcery,
                        flash_surcharge,
                    ) && emit(Action::AnnounceRoom {
                        card,
                        door,
                        flash_surcharge,
                    })
                })
        })
    }

    fn room_in_play(&self, object: ObjectRef) -> Option<RoomDoors> {
        (self.object_ref(object.card) == object)
            .then(|| doors(self, object.card))
            .flatten()
    }

    pub(super) fn set_room_doors(&mut self, object: ObjectRef, doors: RoomDoors) {
        self.forget_room_doors(object);
        if doors != RoomDoors::default() {
            self.state
                .room_openings
                .get_or_insert_default()
                .push((object, doors));
        }
    }

    pub(super) fn open_room_on_entry(&mut self, object: ObjectRef, door: RoomDoor) {
        self.set_room_doors(object, RoomDoors::default().with(door, true));
    }

    pub(super) fn forget_room_doors(&mut self, object: ObjectRef) {
        if let Some(rows) = &mut self.state.room_openings {
            rows.retain(|(key, _)| *key != object);
            if rows.is_empty() {
                self.state.room_openings = None;
            }
        }
    }

    pub(super) fn queue_room_entry_unlock(&mut self, event: &ZoneChange) {
        if event.moved
            && event.before.zone == Zone::Stack
            && event.destination == Zone::Battlefield
            && let Some((_, door)) = event.before.card.room_half()
            && self
                .room_in_play(event.after)
                .is_some_and(|doors| doors.is_open(door))
        {
            self.queue_room_unlock(
                event.after,
                door,
                false,
                self.state.objects[event.after.card as usize].controller,
            );
        }
    }

    fn queue_room_unlock(
        &mut self,
        object: ObjectRef,
        door: RoomDoor,
        fully_open: bool,
        player: Player,
    ) {
        let source = self.object_view(object.card);
        for (index, ability) in source.card.triggers().iter().enumerate() {
            if ability.condition.event() == TriggerCondition::SelfDoorUnlocked(door) {
                self.queue_trigger(&source, index);
            }
        }
        if fully_open {
            let ids: Vec<_> = self
                .state
                .objects
                .iter()
                .enumerate()
                .filter(|(_, object)| {
                    matches!(object.zone, Zone::Battlefield | Zone::Graveyard)
                        && object.card.triggers().iter().any(|ability| {
                            ability.condition.event() == TriggerCondition::Eerie
                                && (object.zone == Zone::Battlefield
                                    || ability.condition.origin() == Some(TargetZone::Graveyard))
                        })
                })
                .map(|(id, _)| id as CardId)
                .collect();
            for (source, grant) in self.indexed_trigger_sources(&ids) {
                if source.controller != player {
                    continue;
                }
                let definition = grant
                    .as_ref()
                    .map_or(source.card, |grant| grant.granting_card);
                for (index, ability) in definition.triggers().iter().enumerate() {
                    if ability.condition.event() == TriggerCondition::Eerie {
                        self.queue_trigger_from(&source, index, grant.map(Box::new));
                    }
                }
            }
        }
    }

    fn change_room_door(
        &mut self,
        object: ObjectRef,
        door: RoomDoor,
        change: RoomChange,
        player: Player,
    ) {
        let before = self.room_in_play(object).expect("legal Room choice");
        let after = before.with(door, change == RoomChange::Unlock);
        assert_ne!(before, after, "a door changes designation once");
        let control_watch = self.freeze_control_watch();
        // A printed ability uses this object's existing timestamp (CR613.7a, CR709.5).
        self.set_room_doors(object, after);
        self.refresh_controllers_after(Some(control_watch));
        if change == RoomChange::Unlock {
            self.queue_room_unlock(object, door, !before.complete() && after.complete(), player);
        }
    }

    pub(super) fn visit_room_unlocks(
        &self,
        player: Player,
        object: ObjectRef,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let Some(doors) = self.room_in_play(object) else {
            return false;
        };
        let definition = self.state.objects[object.card as usize].card;
        let room = definition.room().expect("Room definition");
        let reduction = self
            .state
            .static_sources
            .iter()
            .copied()
            .filter(|&id| {
                self.on_battlefield(id) && self.state.objects[id as usize].controller == player
            })
            .flat_map(|id| {
                self.state.objects[id as usize]
                    .card
                    .static_ability_groups()
                    .iter()
                    .flat_map(move |group| {
                        group
                            .fragments
                            .iter()
                            .filter_map(move |fragment| match *fragment {
                                StaticAbilityFragment::RoomUnlockReduction(amount)
                                    if self.continuous_fragment_available(id, |candidate| {
                                        std::ptr::eq(candidate, fragment)
                                    }) =>
                                {
                                    Some(amount)
                                }
                                _ => None,
                            })
                    })
            })
            .fold(0, u32::saturating_add);
        RoomDoor::ALL
            .into_iter()
            .filter(|&door| !doors.is_open(door))
            .any(|door| {
                room.halves[door.index()].cost_options().any(|mut cost| {
                    cost.generic = cost.generic.saturating_sub(reduction);
                    let request = RoomUnlockPaymentView {
                        object,
                        door,
                        controller: player,
                        cost,
                        definition,
                    };
                    self.can_fund_room_unlock(request)
                        && emit(Action::BeginRoomUnlock { object, door, cost })
                })
            })
    }

    pub(super) fn begin_room_unlock(
        &mut self,
        player: Player,
        object: ObjectRef,
        door: RoomDoor,
        cost: ManaCost,
    ) {
        self.state.room_unlock_payment = Some(RoomUnlockPaymentView {
            object,
            door,
            controller: player,
            cost,
            definition: self.state.objects[object.card as usize].card,
        });
        self.state.pending = Pending::RoomUnlockPayment { player };
        self.state.passes = 0;
    }

    pub(super) fn finish_room_unlock(&mut self, payment: Mana) {
        let request = self
            .state
            .room_unlock_payment
            .take()
            .expect("Room unlock payment");
        self.spend_mana_for(request.controller, payment, mana_lots::Purpose::UnlockDoor);
        self.state.pending = Pending::Priority;
        self.state.priority = request.controller;
        self.state.passes = 0;
        self.change_room_door(
            request.object,
            request.door,
            RoomChange::Unlock,
            request.controller,
        );
    }

    pub(super) fn start_room_choice(
        &mut self,
        item: &SpellView,
        selection: RoomSelection,
        operation: RoomOperation,
    ) -> bool {
        let target = match selection {
            RoomSelection::Target => {
                let Some(object) = item.targets.iter().find_map(|bound| bound.object) else {
                    return false;
                };
                Some(object)
            }
            RoomSelection::Controlled => None,
        };
        let request = RoomChoiceView {
            player: item.controller,
            target,
            operation,
        };
        if !self.visit_room_choices(request, &mut |_| true) {
            return false;
        }
        self.state.room_choice = Some(request);
        self.state.pending = Pending::RoomChoice {
            player: item.controller,
        };
        true
    }

    pub(super) fn visit_room_choices(
        &self,
        request: RoomChoiceView,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let mut visit = |object: ObjectRef| {
            self.room_in_play(object).is_some_and(|doors| {
                RoomDoor::ALL.into_iter().any(|door| {
                    let change = if doors.is_open(door) {
                        RoomChange::Lock
                    } else {
                        RoomChange::Unlock
                    };
                    request.operation.permits(change)
                        && emit(Action::ChooseRoomDoor {
                            object,
                            door,
                            change,
                        })
                })
            })
        };
        if let Some(target) = request.target {
            visit(target)
        } else {
            self.state.objects.iter().enumerate().any(|(id, object)| {
                object.controller == request.player
                    && object.zone == Zone::Battlefield
                    && self.has_subtype(id as CardId, "Room")
                    && visit(self.object_ref(id as CardId))
            })
        }
    }

    pub(super) fn choose_room_door(
        &mut self,
        object: ObjectRef,
        door: RoomDoor,
        change: RoomChange,
    ) {
        let request = self.state.room_choice.take().expect("Room choice");
        self.state.pending = Pending::Priority;
        self.change_room_door(object, door, change, request.player);
        self.run_program();
    }
}
