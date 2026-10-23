//! Typed top-library permission, live stack predicates and simultaneous blink/control.
use super::*;
impl Game {
    pub(super) fn library_top_creature_counter(&self, player: Player, card: CardId) -> bool {
        let object = &self.state.objects[card as usize];
        object.zone==Zone::Library && object.owner==player && object.card.is_creature()
            && self.state.players[player.index()].library.last()==Some(&card)
            && self.state.static_sources.iter().any(|&source| {
                let provider=&self.state.objects[source as usize];
                provider.zone==Zone::Battlefield && provider.controller==player && self.printed_abilities_available(source)
                    && provider.card.static_ability_groups().iter().flat_map(|g|g.fragments).any(|f|match f {
                        crate::cards::StaticAbilityFragment::PlayLibraryTopSubtypeLandsWithCounter(types)=>types.iter().any(|s|object.card.has_subtype(s)),_=>false,
                    })
            })
    }
    pub(super) fn spell_targets_controlled_artifact_or_creature(
        &self,
        item: &SpellView,
        player: Player,
    ) -> bool {
        item.targets.iter().any(|bound| match bound.target {
            Target::Card(id) => {
                self.same_target_object(*bound)
                    && self.on_battlefield(id)
                    && self.state.objects[id as usize].controller == player
                    && self.current_kinds(id) & 17 != 0
            }
            _ => false,
        })
    }
    pub(super) fn blink_all_targets_immediate(&mut self, item: &SpellView) {
        let mut ids = Vec::new();
        for bound in &item.targets {
            if self.same_target_object(*bound)
                && let Target::Card(id) = bound.target
                && self.on_battlefield(id)
                && !ids.contains(&id)
            {
                ids.push(id);
            }
        }
        let events = self.move_objects_raw(&ids, Zone::Exile);
        let mut cards = events
            .iter()
            .filter(|e| {
                e.moved
                    && matches!(e.destination, Zone::Exile | Zone::Graveyard | Zone::Command)
                    && !e.before.is_token()
            })
            .map(|e| ReturnCardView {
                object: e.after,
                zone: e.destination,
                controller: e.before.owner,
                counter: None,
                tapped: false,
            });
        let link = cards.next().map(|first| ReturnLinkView {
            object: first.object,
            zone: first.zone,
            controller: first.controller,
            counter: None,
            tapped: false,
            additional: cards.collect(),
        });
        self.finish_zone_events(&events, &[]);
        if let Some(link) = link {
            self.after_departure(programs::ProgramContinuation::Return(link));
        }
    }
    pub(super) fn take_opposing_artifacts(&mut self, item: &SpellView) {
        let ids = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, o)| {
                o.zone == Zone::Battlefield
                    && o.controller != item.controller
                    && self.current_kinds(*id as CardId) & 16 != 0
            })
            .map(|(id, _)| id as CardId)
            .collect::<Vec<_>>();
        if ids.is_empty() {
            return;
        }
        for &id in &ids {
            self.ensure_control_base(self.object_ref(id));
        }
        let timestamp = self.next_role_timestamp();
        for &id in &ids {
            let object = self.object_ref(id);
            let effect = TemporaryControlView {
                object,
                controller: item.controller,
                previous_controller: self.state.objects[id as usize].controller,
                expires_turn: self.state.turn,
            };
            self.state
                .temporary_controls
                .get_or_insert_with(|| Box::new(Vec::new()))
                .push(effect);
            self.state
                .controls
                .as_mut()
                .unwrap()
                .effects
                .push(continuous_control::ControlEffect {
                    object,
                    controller: item.controller,
                    timestamp,
                    expires_turn: Some(self.state.turn),
                });
        }
        // One continuous effect takes control simultaneously, including dependencies.
        self.refresh_controllers();
        self.untap_objects(&ids, untap_rules::UntapCause::Effect);
        self.grant_keyword_effects(&ids, item.object, Keywords::HASTE);
    }
}

impl Game {
    pub(super) fn library_permission_counter(
        &self,
        player: Player,
        card: CardId,
        permission: ObjectRef,
    ) -> Option<bool> {
        let object = self.state.objects.get(card as usize)?;
        if object.zone != Zone::Library
            || object.owner != player
            || self.state.players[player.index()].library.last() != Some(&card)
            || object.is_token()
        {
            return None;
        }
        let source = self.state.objects.get(permission.card as usize)?;
        if source.zone != Zone::Battlefield
            || self.object_ref(permission.card) != permission
            || source.controller != player
            || !self.printed_abilities_available(permission.card)
        {
            return None;
        }
        let mut permitted = false;
        let mut counter = false;
        for f in source
            .card
            .static_ability_groups()
            .iter()
            .flat_map(|g| g.fragments)
        {
            match f {
                crate::cards::StaticAbilityFragment::CastCreatureFromLibraryTop
                    if object.card.is_creature() =>
                {
                    permitted = true
                }
                crate::cards::StaticAbilityFragment::PlayLibraryTopSubtypeLandsWithCounter(
                    types,
                ) if object.card.is_land() || types.iter().any(|s| object.card.has_subtype(s)) => {
                    permitted = true;
                    counter = object.card.is_creature();
                }
                _ => {}
            }
        }
        permitted.then_some(counter)
    }
    pub(super) fn visit_library_spell_permissions(
        &self,
        actor: Player,
        sorcery: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let Some(&card) = self.state.players[actor.index()].library.last() else {
            return false;
        };
        let object = &self.state.objects[card as usize];
        if object.card.is_land() || !object.card.is_supported() {
            return false;
        }
        for &source in &self.state.static_sources {
            let permission = self.object_ref(source);
            if self
                .library_permission_counter(actor, card, permission)
                .is_none()
            {
                continue;
            }
            for flash_surcharge in [false, true] {
                if flash_surcharge && object.card.flash_surcharge().is_none() {
                    continue;
                }
                for method in [CastMethod::Normal, CastMethod::Warp] {
                    if method == CastMethod::Warp && object.card.warp_cost().is_none() {
                        continue;
                    }
                    if self.permitted_cast_available(
                        actor,
                        card,
                        method,
                        CastFace::Normal,
                        sorcery,
                        flash_surcharge,
                    ) && emit(Action::AnnounceLibrarySpell {
                        card,
                        permission,
                        method,
                        attacker: None,
                        flash_surcharge,
                    }) {
                        return true;
                    }
                }
                if self.visit_sneak_candidates(actor, card, flash_surcharge, &mut |attacker| {
                    emit(Action::AnnounceLibrarySpell {
                        card,
                        permission,
                        method: CastMethod::Sneak,
                        attacker: Some(attacker),
                        flash_surcharge,
                    })
                }) {
                    return true;
                }
            }
        }
        false
    }
}
