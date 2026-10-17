//! Saga chapters are ordinary exact-source triggered packets, not mutable scripts.
use super::*;
impl Game {
    pub(super) fn lore_counters_added(&mut self, object: ObjectRef, before: u64) {
        if !self.on_battlefield(object.card) || self.object_ref(object.card) != object {
            return;
        }
        let after = self.state.objects[object.card as usize].counters.lore;
        if after <= before {
            return;
        }
        let source = self.object_view(object.card);
        for (index, ability) in source.card.triggers().iter().enumerate() {
            if let TriggerCondition::Chapter { number } = ability.condition
                && before < u64::from(number)
                && u64::from(number) <= after
            {
                self.queue_trigger(&source, index);
            }
        }
    }
    pub(super) fn precombat_lore_counters(&mut self) {
        let placements = self
            .state
            .step_sources
            .iter()
            .filter_map(|&id| {
                let o = &self.state.objects[id as usize];
                if o.zone != Zone::Battlefield
                    || o.controller != self.state.active
                    || o.card.saga_final_chapter().is_none()
                {
                    return None;
                }
                Some(replacements::CounterPlacement {
                    placed_by: o.controller,
                    object: self.object_ref(id),
                    card: o.card,
                    owner: o.owner,
                    controller: o.controller,
                    subtypes: self.granted_subtypes(id),
                    kind: self.current_kinds(id),
                    amount: Counters {
                        lore: 1,
                        ..Counters::default()
                    },
                    effect: false,
                    entry_index: None,
                    private: false,
                })
            })
            .collect();
        self.stage_counter_placements(placements, replacements::ResumeMode::Pending);
    }
    fn is_chapter_packet(item: &SpellView, source: ObjectRef) -> bool {
        item.object == source
            && match item.kind {
                StackKind::TriggeredAbility { ability } => item
                    .trigger_definition()
                    .triggers()
                    .get(ability as usize)
                    .is_some_and(|a| matches!(a.condition, TriggerCondition::Chapter { .. })),
                _ => false,
            }
    }
    pub(super) fn saga_sacrifice_required(&self, id: CardId) -> bool {
        let o = &self.state.objects[id as usize];
        let Some(final_chapter) = o.card.saga_final_chapter() else {
            return false;
        };
        if o.zone != Zone::Battlefield
            || !self.printed_abilities_available(id)
            || o.counters.lore < u64::from(final_chapter)
        {
            return false;
        }
        let source = self.object_ref(id);
        !self
            .state
            .stack
            .iter()
            .chain(self.state.pending_triggers.iter())
            .chain(self.state.ordering_triggers.iter())
            .any(|item| Self::is_chapter_packet(item, source))
            && !self
                .state
                .announcing
                .as_ref()
                .is_some_and(|a| Self::is_chapter_packet(&a.item, source))
            && !self
                .state
                .resolving
                .as_ref()
                .is_some_and(|r| Self::is_chapter_packet(&r.item, source))
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{scenario, spawn};
    use super::*;
    fn named(g: &mut Game, p: Player, name: &str, z: Zone) -> CardId {
        spawn(g, p, Card::from_name(name).unwrap(), z)
    }
    fn setup(p: Player) -> Game {
        let mut g = scenario();
        g.state.active = p;
        g.state.priority = p;
        g.state.players[p.index()].mana = [30; 6];
        g
    }
    fn replay(g: &mut Game, a: Action) {
        let p = g.actor().unwrap();
        let saved = g.snapshot();
        let rev = g.revision();
        let parent = g.clone();
        let mut child = parent.clone();
        drop(parent);
        child.apply(p, a.clone()).unwrap();
        g.apply(p, a.clone()).unwrap();
        assert_eq!(child.snapshot(), g.snapshot());
        g.restore(&saved).unwrap();
        assert_eq!(g.apply_index(p, rev, 0), Err(Error::StaleRevision));
        g.apply(p, a).unwrap();
        assert_eq!(child.snapshot(), g.snapshot());
    }
    fn idle(g: &Game) -> bool {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
            && g.state.announcing.is_none()
            && g.state.resolving.is_none()
    }
    fn pick(g: &mut Game) {
        let a = g
            .legal_actions()
            .into_iter()
            .min_by_key(|a| match a {
                Action::ChooseReplacement { .. }
                | Action::ChooseTarget { .. }
                | Action::Discard { .. }
                | Action::Sacrifice { .. }
                | Action::ChooseSearch { .. } => 0,
                Action::PayCosts { .. }
                | Action::PutTrigger { .. }
                | Action::FinishTargets
                | Action::DeclineEffectPayment => 1,
                Action::Pass => 3,
                Action::Concede => 10000,
                _ => 100,
            })
            .unwrap();
        replay(g, a);
    }
    fn settle(g: &mut Game) {
        g.prepare_triggers();
        for _ in 0..180 {
            if idle(g) {
                return;
            }
            pick(g)
        }
        panic!("unsettled {:?}", g.state.pending)
    }
    fn cast(g: &mut Game, p: Player, name: &str) -> CardId {
        let id = named(g, p, name, Zone::Hand);
        replay(g, Action::AnnounceCast { card: id });
        settle(g);
        id
    }
    #[test]
    fn twelve_complete_sagas_cast_pay_and_every_chapter_resolves_both_seats() {
        for p in [Player::P0, Player::P1] {
            for name in [
                "Summon: Choco/Mog",
                "Summon: Esper Ramuh",
                "Summon: Fat Chocobo",
                "Summon: G.F. Ifrit",
                "Burn, Burn, Tree and Fern",
                "Summon: Anima",
                "Leaves from the Vine",
                "Summon: Primal Garuda",
                "Welcome to Sweettooth",
                "The Huntsman's Redemption",
                "Roar of Endless Song",
                "Summon: Shiva",
            ] {
                let mut g = setup(p);
                let ally = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                let second = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
                g.state.objects[enemy as usize].tapped = true;
                let artifact = named(&mut g, p.other(), "Candy Trail", Zone::Battlefield);
                named(&mut g, p, "Firebending Lesson", Zone::Graveyard);
                named(&mut g, p, "Forest", Zone::Hand);
                named(&mut g, p, "Shock", Zone::Graveyard);
                let id = cast(&mut g, p, name);
                assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 1);
                assert!(g.on_battlefield(id));
                assert_eq!(g.state.objects[id as usize].counters.lore, 1);
                assert!(g.is_modified(id));
                assert!(g.damage_facts(id).counter);
                assert!(g.damage_facts(id).modified);
                if name == "Summon: Choco/Mog" {
                    assert_eq!(g.stats(ally).power, 3);
                    assert_eq!(g.stats(id).power, 3)
                }
                if name == "Summon: Fat Chocobo" {
                    assert!(g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
                        && o.owner == p
                        && o.card.token_info().is_some_and(|t| t.subtypes == "Bird")));
                }
                if name == "Summon: G.F. Ifrit" {
                    assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 1);
                    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
                }
                let final_chapter = g.state.objects[id as usize]
                    .card
                    .saga_final_chapter()
                    .unwrap();
                for chapter in 2..=final_chapter {
                    g.begin_step(Step::Main1);
                    assert_eq!(
                        g.state.objects[id as usize].counters.lore,
                        u64::from(chapter)
                    );
                    assert!(!g.saga_sacrifice_required(id));
                    settle(&mut g);
                    if chapter < final_chapter {
                        assert!(g.on_battlefield(id));
                    }
                }
                assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
                match name {
                    "Burn, Burn, Tree and Fern" => {
                        assert!(!g.on_battlefield(enemy));
                        assert!(!g.on_battlefield(artifact));
                    }
                    "Summon: Anima" => {
                        assert!(!g.on_battlefield(enemy));
                        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
                        assert_eq!(g.state.players[p.index()].life, 17);
                        assert_eq!(g.state.players[p.other().index()].life, 17);
                    }
                    "Leaves from the Vine" => {
                        assert_eq!(g.state.objects[ally as usize].counters.plus_one, 1);
                        assert_eq!(g.state.objects[second as usize].counters.plus_one, 1);
                        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
                    }
                    "Summon: Primal Garuda" => {
                        assert!(!g.on_battlefield(enemy));
                        assert_eq!(g.stats(ally).power, 4);
                        assert!(g.keywords(ally).contains(Keywords::FLYING));
                    }
                    "Welcome to Sweettooth" => {
                        assert_eq!(g.state.objects[ally as usize].counters.plus_one, 2);
                    }
                    "The Huntsman's Redemption" => {
                        assert!(!g.on_battlefield(ally));
                        assert_eq!(g.stats(second).power, 4);
                        assert!(g.keywords(second).contains(Keywords::TRAMPLE));
                        assert_eq!(g.hand_count(p), 2);
                    }
                    "Roar of Endless Song" => {
                        assert_eq!(g.stats(ally).power, 4);
                        assert_eq!(g.stats(second).power, 4);
                    }
                    "Summon: Shiva" => {
                        assert_eq!(g.state.objects[enemy as usize].counters.stun, 2);
                        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
                    }
                    _ => {}
                }
            }
        }
    }
    #[test]
    fn doubling_entry_queues_distinct_thresholds_but_precombat_is_not_an_effect() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            named(&mut g, p, "Doubling Season", Zone::Battlefield);
            let id = named(&mut g, p, "Summon: Choco/Mog", Zone::Hand);
            replay(&mut g, Action::AnnounceCast { card: id });
            for _ in 0..40 {
                if g.on_battlefield(id) {
                    break;
                }
                pick(&mut g);
            }
            assert_eq!(g.state.objects[id as usize].counters.lore, 2);
            let packets = g
                .state
                .pending_triggers
                .iter()
                .chain(g.state.ordering_triggers.iter())
                .chain(g.state.stack.iter())
                .filter(|s| s.object == g.object_ref(id))
                .collect::<Vec<_>>();
            assert_eq!(packets.len(), 2);
            assert_ne!(packets[0].kind, packets[1].kind);
            settle(&mut g);
            g.begin_step(Step::Main2);
            assert_eq!(g.state.objects[id as usize].counters.lore, 2);
            g.begin_step(Step::Main1);
            assert_eq!(g.state.objects[id as usize].counters.lore, 3);
            settle(&mut g);
            assert!(g.on_battlefield(id));
        }
    }
    #[test]
    fn any_outstanding_chapter_holds_sba_and_old_incarnation_does_not_hold_new_source() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            let id = named(&mut g, p, "Summon: Choco/Mog", Zone::Hand);
            g.move_object(id, Zone::Battlefield);
            let old = g.object_ref(id);
            g.state.objects[id as usize].counters.lore = 4;
            assert!(!g.saga_sacrifice_required(id));
            g.prepare_triggers();
            assert!(!g.saga_sacrifice_required(id));
            assert!(g.state.stack.iter().any(|s| s.object == old));
            g.move_object(id, Zone::Hand);
            g.move_object(id, Zone::Battlefield);
            assert_ne!(g.object_ref(id), old);
            g.state.objects[id as usize].counters.lore = 4;
            g.state.pending_triggers.clear();
            g.state.ordering_triggers.clear();
            assert!(g.saga_sacrifice_required(id));
            g.state_based_actions();
            assert!(!g.on_battlefield(id));
            settle(&mut g);
        }
    }
    #[test]
    fn lore_only_modified_creature_deals_actual_combat_damage_and_qualification_survives_departure()
    {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            named(&mut g, p, "SP//dr, Piloted by Peni", Zone::Battlefield);
            named(&mut g, p, "Yathan Tombguard", Zone::Battlefield);
            let source = cast(&mut g, p, "Summon: Choco/Mog");
            assert_eq!(g.state.objects[source as usize].counters.total(), 1);
            assert_eq!(g.state.objects[source as usize].counters.lore, 1);
            g.state.objects[source as usize].summoning_sick = false;
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            replay(&mut g, Action::Attack { card: source });
            replay(&mut g, Action::FinishAttackers);
            replay(&mut g, Action::Pass);
            replay(&mut g, Action::Pass);
            replay(&mut g, Action::FinishBlockers);
            replay(&mut g, Action::Pass);
            replay(&mut g, Action::Pass);
            assert_eq!(g.state.players[p.other().index()].life, 17);
            let drawn = g.state.players[p.index()].cards_drawn_this_turn;
            let life = g.state.players[p.index()].life;
            let old = g.object_ref(source);
            assert!(
                g.state
                    .stack
                    .iter()
                    .chain(g.state.ordering_triggers.iter())
                    .any(|item| item.damage_event.is_some_and(|e| e.dealer == Some(old)))
            );
            g.move_object(source, Zone::Hand);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, drawn + 2);
            assert_eq!(g.state.players[p.index()].life, life - 1);
            assert_eq!(g.state.objects[source as usize].counters.total(), 0);
        }
    }
    #[test]
    fn saga_memory_and_counter_wire_values_are_explicit() {
        let g = Game::new(Config {
            decks: [vec![Card::Forest; 60], vec![Card::Forest; 60]],
            seed: 256,
            starting_player: Player::P0,
            deck_rules: DeckRules::Freeform,
        })
        .unwrap();
        eprintln!(
            "Saga core layout: Counters={} Object={} State={} Announcement={} SpellView={} Resolution={} EntryMove={} CounterPlacement={} initial={}",
            std::mem::size_of::<Counters>(),
            std::mem::size_of::<Object>(),
            std::mem::size_of::<State>(),
            std::mem::size_of::<Announcement>(),
            std::mem::size_of::<SpellView>(),
            std::mem::size_of::<Resolution>(),
            std::mem::size_of::<entries::EntryMove>(),
            std::mem::size_of::<replacements::CounterPlacement>(),
            g.state_bytes()
        );
        let counters = Counters {
            lore: u64::MAX,
            ..Counters::default()
        };
        let text = serde_json::to_string(&counters).unwrap();
        assert_eq!(serde_json::from_str::<Counters>(&text).unwrap(), counters);
        assert_eq!(counters.total(), u128::from(u64::MAX));
    }
    #[test]
    fn lore_removal_and_readdition_retriggers_and_overflow_is_atomic() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            let id = cast(&mut g, p, "Summon: Choco/Mog");
            g.state.objects[id as usize].counters.lore = 0;
            let reference = g.object_ref(id);
            g.effect_counter(id, CounterKind::Lore, 2, replacements::ResumeMode::Pending);
            assert_eq!(
                g.state
                    .pending_triggers
                    .iter()
                    .filter(|s| s.object == reference)
                    .count(),
                2
            );
            settle(&mut g);
            assert_eq!(g.state.objects[id as usize].counters.lore, 2);
            // A real counter-doubling activation above an older chapter must fail atomically.
            let mut g = setup(p);
            let z = named(&mut g, p, "Zimone, Paradox Sculptor", Zone::Battlefield);
            let id = named(&mut g, p, "Summon: Choco/Mog", Zone::Hand);
            g.move_object(id, Zone::Battlefield);
            g.state.objects[id as usize].counters.lore = u64::MAX;
            g.prepare_triggers();
            replay(
                &mut g,
                Action::AnnounceActivate {
                    card: z,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            replay(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
            replay(&mut g, Action::FinishTargets);
            replay(
                &mut g,
                Action::PayCosts {
                    payment: [0, 1, 0, 0, 1, 0],
                },
            );
            replay(&mut g, Action::Pass);
            let before = g.snapshot();
            let revision = g.revision();
            let actor = g.actor().unwrap();
            assert!(matches!(
                g.apply(actor, Action::Pass),
                Err(Error::CapacityExceeded(_))
            ));
            assert_eq!(g.snapshot(), before);
            assert_eq!(g.revision(), revision);
            assert!(g.state.objects[z as usize].tapped);
            assert_eq!(g.state.objects[id as usize].counters.lore, u64::MAX);
        }
    }

    fn chapter_number(item: &SpellView) -> u16 {
        let StackKind::TriggeredAbility { ability } = item.kind else {
            panic!("not trigger")
        };
        let TriggerCondition::Chapter { number } =
            item.trigger_definition().triggers()[ability as usize].condition
        else {
            panic!("not chapter")
        };
        number
    }
    #[test]
    fn final_chapter_leaving_stack_does_not_sacrifice_while_earlier_chapters_remain() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            let id = cast(&mut g, p, "Summon: Choco/Mog");
            g.effect_counter(id, CounterKind::Lore, 3, replacements::ResumeMode::Pending);
            g.prepare_triggers();
            while !g.state.ordering_triggers.is_empty() {
                let packet = g
                    .state
                    .ordering_triggers
                    .iter()
                    .min_by_key(|v| chapter_number(v))
                    .unwrap()
                    .id;
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a,Action::PutTrigger{id}if *id==packet))
                    .unwrap();
                replay(&mut g, a);
            }
            assert_eq!(chapter_number(g.state.stack.last().unwrap()), 4);
            replay(&mut g, Action::Pass);
            replay(&mut g, Action::Pass);
            assert!(g.on_battlefield(id));
            assert!(!g.saga_sacrifice_required(id));
            assert!(g.state.stack.iter().all(|item| chapter_number(item) < 4));
            // No printed admitted spell currently counters an ability; exercise the shared native counter operation.
            let packet = g.state.stack.last().unwrap().id;
            assert!(g.counter_stack_item(packet, false));
            g.state_based_actions();
            assert!(g.on_battlefield(id));
            let packet = g.state.stack.last().unwrap().id;
            assert!(g.counter_stack_item(packet, false));
            g.state_based_actions();
            assert!(!g.on_battlefield(id));
        }
    }
    #[test]
    fn final_anima_resolution_owned_sacrifice_keeps_source_and_parent_closed_packet() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let source = cast(&mut g, p, "Summon: Anima");
            g.effect_counter(
                source,
                CounterKind::Lore,
                3,
                replacements::ResumeMode::Pending,
            );
            g.prepare_triggers();
            while !g.state.ordering_triggers.is_empty() {
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PutTrigger { .. }))
                    .unwrap();
                replay(&mut g, a);
            }
            let lower: Vec<_> = g
                .state
                .stack
                .iter()
                .filter(|v| chapter_number(v) != 4)
                .map(|v| v.id)
                .collect();
            for id in lower {
                assert!(g.counter_stack_item(id, false));
            }
            replay(&mut g, Action::Pass);
            replay(&mut g, Action::Pass);
            assert!(g.state.resolving.is_some());
            assert_eq!(g.actor(), Some(p.other()));
            assert!(g.on_battlefield(source));
            assert!(!g.saga_sacrifice_required(source));
            assert!(g.observe(p).effect_choice.is_some());
            assert!(g.observe(p).actions.is_empty());
            let a = Action::Sacrifice {
                object: g.object_ref(enemy),
            };
            let old = g.snapshot();
            let rev = g.revision();
            assert_eq!(g.apply(p, a.clone()), Err(Error::WrongActor));
            assert_eq!(g.snapshot(), old);
            assert_eq!(g.revision(), rev);
            replay(&mut g, a);
            settle(&mut g);
            assert!(!g.on_battlefield(enemy));
            assert!(!g.on_battlefield(source));
            assert_eq!(g.state.players[p.other().index()].life, 17);
        }
    }
    #[test]
    fn first_turn_skipped_draw_main_only_and_simultaneous_entry_lore_is_apnap() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            let source = cast(&mut g, p, "Summon: Choco/Mog");
            g.state.turn = 1;
            g.state.step = Step::Upkeep;
            g.give_priority();
            replay(&mut g, Action::Pass);
            replay(&mut g, Action::Pass);
            assert_eq!(g.state.step, Step::Main1);
            assert_eq!(g.state.objects[source as usize].counters.lore, 2);
            settle(&mut g);
            let mut g = setup(p);
            named(&mut g, p, "Doubling Season", Zone::Battlefield);
            named(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
            let a = named(&mut g, p, "Summon: Choco/Mog", Zone::Hand);
            let b = named(&mut g, p.other(), "Summon: Fat Chocobo", Zone::Hand);
            g.stage_entries(
                vec![
                    g.entry_move(b, Zone::Battlefield, false),
                    g.entry_move(a, Zone::Battlefield, false),
                ],
                None,
            );
            g.advance_entries();
            assert_eq!(g.state.objects[a as usize].counters.lore, 2);
            assert_eq!(g.state.objects[b as usize].counters.lore, 2);
            assert_eq!(g.state.pending_triggers.len(), 4);
            g.prepare_triggers();
            assert_eq!(g.actor(), Some(p));
            assert!(
                g.state
                    .ordering_triggers
                    .iter()
                    .any(|s| s.controller == p.other())
            );
        }
    }
    #[test]
    fn leaves_guard_is_or_and_roar_captures_all_signed_operands() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            let leaves = cast(&mut g, p, "Leaves from the Vine");
            g.state
                .objects
                .iter_mut()
                .filter(|o| o.zone == Zone::Graveyard)
                .for_each(|o| o.zone = Zone::Exile);
            named(&mut g, p, "Firebending Lesson", Zone::Graveyard);
            let before = g.state.players[p.index()].cards_drawn_this_turn;
            g.effect_counter(
                leaves,
                CounterKind::Lore,
                2,
                replacements::ResumeMode::Pending,
            );
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, before + 1);
            // A temporary group effect captures current signed values. Later entrants and changes don't rewrite it.
            let mut g = setup(p);
            let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.objects[b as usize].growth.power = -3; // -1/2 survives SBAs; power must remain signed.
            let roar = cast(&mut g, p, "Roar of Endless Song");
            g.effect_counter(
                roar,
                CounterKind::Lore,
                2,
                replacements::ResumeMode::Pending,
            );
            settle(&mut g);
            assert_eq!(
                g.stats(a),
                Stats {
                    power: 4,
                    toughness: 4
                }
            );
            assert_eq!(
                g.stats(b),
                Stats {
                    power: -2,
                    toughness: 4
                }
            );
            let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            assert_eq!(g.stats(c).power, 2);
            g.state.objects[a as usize].counters.plus_one = 1;
            assert_eq!(g.stats(a).power, 5);
        }
    }

    #[test]
    fn shiva_draw_counts_current_tapped_opposing_animated_vehicles_not_own_or_untapped() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            let vehicle = named(&mut g, p.other(), "Cultivator's Caravan", Zone::Battlefield);
            let first = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let crew = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.objects[own as usize].tapped = true;
            let source = cast(&mut g, p, "Summon: Shiva");
            assert!(g.state.objects[first as usize].tapped);
            g.state.priority = p.other();
            let ability = g.state.objects[vehicle as usize]
                .card
                .activations()
                .iter()
                .position(|a| a.crew_minimum().is_some())
                .unwrap() as u16;
            replay(
                &mut g,
                Action::AnnounceActivate {
                    card: vehicle,
                    ability,
                    counter_payment: Counters::default(),
                },
            );
            let contributor = g.object_ref(crew);
            replay(
                &mut g,
                Action::ChooseCrewer {
                    object: contributor,
                },
            );
            replay(&mut g, Action::FinishCrew);
            settle(&mut g);
            assert!(g.is_creature(vehicle));
            g.state.priority = p.other();
            let mana = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a,Action::ActivateMana{card,..}if *card==vehicle))
                .unwrap();
            replay(&mut g, mana);
            assert!(g.state.objects[vehicle as usize].tapped);
            let before = g.state.players[p.index()].cards_drawn_this_turn;
            g.effect_counter(
                source,
                CounterKind::Lore,
                2,
                replacements::ResumeMode::Pending,
            );
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, before + 3);
        }
    }
    #[test]
    fn saga_sba_has_real_sacrifice_cause_and_lore_is_available_for_generic_counter_costs() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            let watcher = named(&mut g, p, "Pirate Peddlers", Zone::Battlefield);
            let source = cast(&mut g, p, "Summon: Choco/Mog");
            let mut payments = Vec::new();
            let cost = Card::from_name("Brambleback Brute")
                .unwrap()
                .activations()
                .iter()
                .find(|a| a.cost.remove == Some((None, 1)))
                .unwrap()
                .cost;
            super::super::activations::counter_payments(
                cost,
                &g.state.objects[source as usize].counters,
                |p| {
                    payments.push(p);
                    false
                },
            );
            assert_eq!(
                payments,
                vec![Counters {
                    lore: 1,
                    ..Counters::default()
                }]
            );
            g.effect_counter(
                source,
                CounterKind::Lore,
                3,
                replacements::ResumeMode::Pending,
            );
            settle(&mut g);
            assert_eq!(g.state.objects[watcher as usize].counters.plus_one, 1);
            assert!(!g.on_battlefield(source));
        }
    }
}
