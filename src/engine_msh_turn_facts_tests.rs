//! Paid whole-card scenarios and explicitly bounded effective-characteristic fixtures.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::msh_galactus_tests::{reject, until};
use super::*;
const IRON: &str = "Iron Man, Master of Machines";
const AVENGERS: &str = "Avengers Assemble!";
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    g.state.players[p.index()].mana = [30; 6];
    super::msh_galactus_tests::paid(g, p, name, targets)
}
fn treasure() -> Card {
    Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Treasure"))
        .unwrap()
}
fn facts(g: &Game, p: Player) -> TurnFactsView {
    g.state.players[p.index()].turn_facts
}
fn iron(g: &mut Game, p: Player) -> CardId {
    paid(g, p, IRON, &[])
}
fn end(g: &mut Game) {
    g.begin_step(Step::End);
    settle(g);
}
fn attack(g: &mut Game, id: CardId) {
    until(g, |g| g.state.pending == Pending::Attackers);
    checked(g, Action::Attack { card: id });
    checked(g, Action::FinishAttackers);
}
fn queued(g: &Game, id: CardId) -> bool {
    g.state
        .pending_triggers
        .iter()
        .chain(&g.state.stack)
        .chain(&g.state.ordering_triggers)
        .any(|i| i.card == id && matches!(i.kind, StackKind::TriggeredAbility { .. }))
}
fn types(g: &mut Game, id: CardId, names: &'static [&'static str]) {
    let recipient = g.object_ref(id);
    let timestamp = g.next_role_timestamp();
    g.state
        .roles
        .get_or_insert_default()
        .creature_types
        .push(subtypes::CreatureTypesEffectView {
            recipient,
            source: recipient,
            types: names,
            additional: false,
            timestamp,
        });
}
#[test]
fn msh_turn_facts_iron_paid_cost_keywords_and_illegal_cost_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, IRON, Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 3];
        reject(&mut g, Action::AnnounceCast { card: id });
        g.state.players[p.index()].mana = [0, 1, 0, 1, 0, 2];
        cast(&mut g, id, None, &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!((g.stats(id).power, g.stats(id).toughness), (1, 4));
        assert!(
            g.keywords(id)
                .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
        );
        assert!(facts(&g, p).artifact_entered && facts(&g, p).hero_entered);
    }
}
#[test]
fn msh_turn_facts_iron_self_entry_paid_haste_attack_draw_and_vigilance() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = iron(&mut g, p);
        paid(&mut g, p, "Impolite Entrance", &[Target::Card(id)]);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let before = g.hand_count(p);
        attack(&mut g, id);
        assert!(!g.object_view(id).tapped);
        settle(&mut g);
        assert_eq!(g.hand_count(p), before + 1);
        assert!(facts(&g, p).hero_attacked);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        reject(
            &mut g,
            Action::Block {
                blocker: bear,
                attacker: id,
            },
        );
    }
}
#[test]
fn msh_turn_facts_iron_old_source_no_entry_and_late_artifact_no_retroactive_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = iron(&mut g, p);
        next_turn(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.step == Step::Main1 && g.state.pending == Pending::Priority
        });
        assert!(!facts(&g, p).artifact_entered);
        let before = g.hand_count(p);
        attack(&mut g, id);
        assert!(!queued(&g, id));
        g.create_tokens(p, treasure(), 1, false);
        settle(&mut g);
        assert_eq!(g.hand_count(p), before);
    }
}
#[test]
fn msh_turn_facts_iron_prior_entry_survives_departure_and_control_transfer() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = iron(&mut g, p);
        next_turn(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.step == Step::Main1 && g.state.pending == Pending::Priority
        });
        let a = paid(&mut g, p, "Hedron Archive", &[]);
        g.change_controller(g.object_ref(a), p.other());
        g.move_object(a, Zone::Graveyard);
        let before = g.hand_count(p);
        attack(&mut g, id);
        settle(&mut g);
        assert_eq!(g.hand_count(p), before + 1);
    }
}
#[test]
fn msh_turn_facts_iron_queued_source_departure_keeps_captured_player_and_view() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = iron(&mut g, p);
        paid(&mut g, p, "Impolite Entrance", &[Target::Card(id)]);
        attack(&mut g, id);
        assert!(queued(&g, id));
        g.change_controller(g.object_ref(id), p.other());
        g.move_object(id, Zone::Hand);
        assert!(g.observe(p).turn_facts.is_some());
        let before = g.hand_count(p);
        settle(&mut g);
        assert_eq!(g.hand_count(p), before + 1);
        assert!(g.observe(p).turn_facts.is_none());
    }
}
#[test]
fn msh_turn_facts_iron_live_other_artifact_count_is_additive_not_cda() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = iron(&mut g, p);
        let a = paid(&mut g, p, "Hedron Archive", &[]);
        let opponent = named(&mut g, p.other(), "Hedron Archive", Zone::Battlefield);
        assert_eq!(g.stats(id).power, 2);
        g.create_tokens(p, treasure(), 2, false);
        assert_eq!(g.stats(id).power, 4);
        g.change_controller(g.object_ref(a), p.other());
        assert_eq!(g.stats(id).power, 3);
        g.change_controller(g.object_ref(opponent), p);
        assert_eq!(g.stats(id).power, 4);
        paid(&mut g, p, "I Am Iron Man", &[Target::Card(id)]);
        assert_eq!(g.stats(id).power, 7);
    }
}
#[test]
fn msh_turn_facts_paid_molten_copy_is_artifact_hero_entry_and_public_observer() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = iron(&mut g, p);
        let harvester = paid(&mut g, p, "Barkform Harvester", &[]);
        g.state.players[p.index()].turn_facts = TurnFactsView::default();
        paid(&mut g, p, "Molten Duplication", &[Target::Card(harvester)]);
        assert!(facts(&g, p).artifact_entered && facts(&g, p).hero_entered);
        assert!(g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
            && o.is_copy_token()
            && o.card == Card::from_name("Barkform Harvester").unwrap()));
        assert_eq!(g.stats(id).power, 3);
    }
}
#[test]
fn msh_turn_facts_avengers_paid_flash_after_prior_departed_hero_entry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        // The prior Hero really enters during the opponent's turn through paid flash.
        g.state.active = p.other();
        g.state.priority = p;
        let id = paid(&mut g, p, "Agent Venom", &[]);
        g.move_object(id, Zone::Graveyard);
        assert!(g.observe(p).turn_facts.is_none());
        g.state.active = p.other();
        g.state.priority = p;
        g.state.step = Step::Main1;
        let av = paid(&mut g, p, AVENGERS, &[]);
        assert!(g.on_battlefield(av));
        let before = g.hand_count(p);
        end(&mut g);
        assert_eq!(g.hand_count(p), before + 1);
    }
}
#[test]
fn msh_turn_facts_avengers_after_hero_attack_and_death_without_entry_fact() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let hero = named(&mut g, p, "Kid Loki", Zone::Battlefield);
        g.state.objects[hero as usize].summoning_sick = false;
        g.state.entered_this_turn = None; // Explicit older-source fixture; no entry event this turn.
        attack(&mut g, hero);
        settle(&mut g);
        g.move_object(hero, Zone::Graveyard);
        assert!(!facts(&g, p).hero_entered && facts(&g, p).hero_attacked);
        // Bounded priority location after combat: flash still pays and uses actual stored attack.
        g.state.pending = Pending::Priority;
        g.state.priority = p;
        paid(&mut g, p, AVENGERS, &[]);
        let before = g.hand_count(p);
        end(&mut g);
        assert_eq!(g.hand_count(p), before + 1);
    }
}
#[test]
fn msh_turn_facts_avengers_no_fact_no_trigger_and_entry_after_end_is_late() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let av = paid(&mut g, p, AVENGERS, &[]);
        g.begin_step(Step::End);
        assert!(!queued(&g, av));
        let hero = named(&mut g, p, "Kid Loki", Zone::Hand);
        g.stage_entries(vec![g.entry_move(hero, Zone::Battlefield, false)], None);
        settle(&mut g);
        assert!(facts(&g, p).hero_entered);
        assert!(!queued(&g, av));
    }
}
#[test]
fn msh_turn_facts_avengers_enters_after_end_begins_no_retroactive_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        iron(&mut g, p);
        g.begin_step(Step::End);
        let av = paid(&mut g, p, AVENGERS, &[]);
        assert!(!queued(&g, av));
    }
}
#[test]
fn msh_turn_facts_avengers_or_is_one_draw_multiple_sources_each_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = iron(&mut g, p);
        paid(&mut g, p, "Impolite Entrance", &[Target::Card(id)]);
        attack(&mut g, id);
        settle(&mut g);
        g.state.pending = Pending::Priority;
        g.state.priority = p;
        paid(&mut g, p, AVENGERS, &[]);
        paid(&mut g, p, AVENGERS, &[]);
        let before = g.hand_count(p);
        end(&mut g);
        assert_eq!(g.hand_count(p), before + 2);
    }
}
#[test]
fn msh_turn_facts_avengers_lord_live_hero_types_controls_and_source_leave() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let hero = named(&mut g, p, "Kid Loki", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let av = paid(&mut g, p, AVENGERS, &[]);
        assert_eq!((g.stats(hero).power, g.stats(hero).toughness), (3, 3));
        assert_eq!(g.stats(bear).power, 2);
        types(&mut g, bear, &["Hero"]);
        assert_eq!(g.stats(bear).power, 4);
        g.change_controller(g.object_ref(hero), p.other());
        assert_eq!(g.stats(hero).power, 1);
        g.move_object(av, Zone::Graveyard);
        assert_eq!(g.stats(bear).power, 2);
    }
}
#[test]
fn msh_turn_facts_avengers_captured_controller_survives_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        iron(&mut g, p);
        let av = paid(&mut g, p, AVENGERS, &[]);
        g.begin_step(Step::End);
        assert!(queued(&g, av));
        g.change_controller(g.object_ref(av), p.other());
        g.move_object(av, Zone::Hand);
        let before = g.hand_count(p);
        settle(&mut g);
        assert_eq!(g.hand_count(p), before + 1);
    }
}
#[test]
fn msh_turn_facts_controller_at_event_does_not_borrow_previous_controllers_history() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        iron(&mut g, p);
        let av = paid(&mut g, p, AVENGERS, &[]);
        g.change_controller(g.object_ref(av), p.other());
        g.begin_step(Step::End);
        assert!(!queued(&g, av));
        assert!(!facts(&g, p.other()).hero_entered);
    }
}
#[test]
fn msh_turn_facts_public_optin_never_tests_hidden_hand_or_library() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let hidden = named(&mut g, p, AVENGERS, Zone::Hand);
        let before = serde_json::to_value(g.observe(p.other())).unwrap();
        g.state.objects[hidden as usize].card = Card::from_name("Bear Cub").unwrap();
        assert_eq!(serde_json::to_value(g.observe(p.other())).unwrap(), before);
        g.state.players[p.index()].turn_facts = TurnFactsView {
            artifact_entered: true,
            hero_entered: true,
            hero_attacked: true,
        };
        assert_eq!(serde_json::to_value(g.observe(p.other())).unwrap(), before);
        let av = paid(&mut g, p, AVENGERS, &[]);
        let a = g.observe(p).turn_facts.unwrap();
        assert_eq!(g.observe(p.other()).turn_facts, Some(a));
        g.move_object(av, Zone::Hand);
        assert!(g.observe(p).turn_facts.is_none());
    }
}
#[test]
fn msh_turn_facts_orphan_raw_indexed_end_trigger_restore_full_observations() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        iron(&mut g, p);
        paid(&mut g, p, AVENGERS, &[]);
        g.begin_step(Step::End);
        g.prepare_triggers();
        let actor = g.actor().unwrap();
        let revision = g.revision();
        let index = g
            .legal_actions()
            .iter()
            .position(|a| *a == Action::Pass)
            .unwrap();
        let snapshot = g.snapshot();
        let bytes = g.state_bytes();
        let views = [
            serde_json::to_value(g.observe(p)).unwrap(),
            serde_json::to_value(g.observe(p.other())).unwrap(),
        ];
        let mut raw = {
            let parent = g.clone();
            parent.clone()
        };
        let mut indexed = raw.clone();
        drop(g);
        raw.apply(actor, Action::Pass).unwrap();
        indexed.apply_index(actor, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        raw.restore(&snapshot).unwrap();
        assert_eq!(raw.state_bytes(), bytes);
        let mut restored = [
            serde_json::to_value(raw.observe(p)).unwrap(),
            serde_json::to_value(raw.observe(p.other())).unwrap(),
        ];
        // Restore intentionally advances the protocol revision, while all rule/private fields restore exactly.
        for (after, before) in restored.iter_mut().zip(&views) {
            after["revision"] = before["revision"].clone();
        }
        assert_eq!(restored, views);
    }
}
#[test]
fn msh_turn_facts_new_turn_resets_both_players_not_source_departures() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        iron(&mut g, p);
        g.create_tokens(p.other(), treasure(), 1, false);
        assert!(facts(&g, p).artifact_entered && facts(&g, p.other()).artifact_entered);
        next_turn(&mut g);
        assert_eq!(facts(&g, p), TurnFactsView::default());
        assert_eq!(facts(&g, p.other()), TurnFactsView::default());
    }
}
#[test]
fn msh_turn_facts_bounded_noncreature_kindred_changeling_entry_qualifies() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Hedron Archive", Zone::Hand);
        // Only gated immutable metadata is used; Firdoch Core is not admitted/playable.
        g.state.objects[id as usize].card = Card::from_name("Firdoch Core").unwrap();
        g.stage_entries(vec![g.entry_move(id, Zone::Battlefield, false)], None);
        settle(&mut g);
        assert!(facts(&g, p).hero_entered && facts(&g, p).artifact_entered);
        assert!(!g.is_creature(id));
    }
}
#[test]
fn msh_turn_facts_simultaneous_entries_raw_return_and_artifact_land_fixture() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = named(&mut g, p, IRON, Zone::Graveyard);
        let b = named(&mut g, p, "Hedron Archive", Zone::Hand);
        g.stage_entries(
            vec![
                g.entry_move(a, Zone::Battlefield, false),
                g.entry_move(b, Zone::Battlefield, false),
            ],
            None,
        );
        settle(&mut g);
        assert!(facts(&g, p).artifact_entered && facts(&g, p).hero_entered);
        g.move_object(a, Zone::Hand);
        g.state.players[p.index()].turn_facts = TurnFactsView::default();
        g.move_object(a, Zone::Battlefield);
        assert!(facts(&g, p).hero_entered);
        // Bounded printed artifact-land metadata; not a newly admitted artifact-land producer.
        let land = named(&mut g, p, "Forest", Zone::Hand);
        g.state.objects[land as usize].card = Card::from_name("Scene of the Crime").unwrap();
        g.state.players[p.index()].turn_facts = TurnFactsView::default();
        g.move_object(land, Zone::Battlefield);
        assert!(facts(&g, p).artifact_entered);
        assert_eq!(g.stats(a).power, 3); // Archive plus the artifact land, excluding self.
    }
}
#[test]
fn msh_turn_facts_zero_tokens_and_impossible_aura_do_not_count_entry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.create_tokens(p, treasure(), 0, false);
        assert_eq!(facts(&g, p), TurnFactsView::default());
        let aura = named(&mut g, p, "Confiscate", Zone::Graveyard);
        g.stage_entries(vec![g.entry_move(aura, Zone::Battlefield, false)], None);
        g.advance_entries();
        assert_eq!(facts(&g, p), TurnFactsView::default());
        assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_turn_facts_bounded_posttap_effective_type_and_put_attacking_distinction() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let hero = named(&mut g, p, "Kid Loki", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.attackers.push(Attacker {
            card: hero,
            defender: Some(CombatDefender::Player(p.other())),
            blocked: false,
        });
        g.tap_objects(&[hero]);
        types(&mut g, hero, &["Bear"]);
        // Bounded earlier-layer result after tapping: not an admitted tap/type producer.
        let declared = [AttackDeclaration {
            blocked: false,
            card: hero,
            defender: Some(CombatDefender::Player(p.other())),
        }];
        g.record_attack_turn_facts(p, &declared);
        assert!(!facts(&g, p).hero_attacked);
        types(&mut g, hero, &["Hero"]);
        g.record_attack_turn_facts(p, &declared);
        assert!(facts(&g, p).hero_attacked);
        g.state.players[p.index()].turn_facts = TurnFactsView::default();
        g.state.attackers.push(Attacker {
            card: bear,
            defender: Some(CombatDefender::Player(p.other())),
            blocked: false,
        });
        types(&mut g, bear, &["Hero"]);
        assert!(!facts(&g, p).hero_attacked); // Entering/being attacking alone is never an attack event.
    }
}

#[test]
fn msh_turn_facts_iron_paid_from_actual_land_funding() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let lands = ["Island", "Mountain", "Forest", "Forest"]
            .map(|n| named(&mut g, p, n, Zone::Battlefield));
        let id = named(&mut g, p, IRON, Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        for &land in &lands {
            let action = Action::TapMana { card: land };
            checked(&mut g, action);
        }
        cast(&mut g, id, None, &[]);
        settle(&mut g);
        assert!(g.on_battlefield(id));
        assert!(lands.iter().all(|id| g.object_view(*id).tapped));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn msh_turn_facts_avengers_flash_from_actual_land_funding() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        // The prior Hero really enters during the opponent's turn through paid flash.
        g.state.active = p.other();
        g.state.priority = p;
        let id = paid(&mut g, p, "Agent Venom", &[]);
        g.move_object(id, Zone::Graveyard);
        let lands = [0; 5].map(|_| named(&mut g, p, "Plains", Zone::Battlefield));
        let av = named(&mut g, p, AVENGERS, Zone::Hand);
        g.state.active = p.other();
        g.state.priority = p;
        g.state.players[p.index()].mana = [0; 6];
        for &land in &lands {
            let action = Action::TapMana { card: land };
            checked(&mut g, action);
        }
        cast(&mut g, av, None, &[]);
        settle(&mut g);
        assert!(lands.iter().all(|id| g.object_view(*id).tapped));
        let before = g.hand_count(p);
        end(&mut g);
        assert_eq!(g.hand_count(p), before + 1);
    }
}
#[test]
fn msh_turn_facts_avengers_lord_enables_actual_crew_four_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let hero = named(&mut g, p, "Human Torch, Johnny Storm", Zone::Battlefield);
        let vehicle = named(&mut g, p, "Dependable Quinjet", Zone::Battlefield);
        assert_eq!(g.stats(hero).power, 2);
        paid(&mut g, p, AVENGERS, &[]);
        assert_eq!(g.stats(hero).power, 4);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: vehicle,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        let object = g.object_ref(hero);
        checked(&mut g, Action::ChooseCrewer { object });
        checked(&mut g, Action::FinishCrew);
        super::msh_galactus_tests::finish(&mut g);
        settle(&mut g);
        assert!(g.object_view(hero).tapped);
        assert!(g.is_creature(vehicle));
    }
}
#[test]
fn msh_turn_facts_suppressed_public_observer_stays_visible_but_no_lord_or_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let hero = iron(&mut g, p);
        let av = paid(&mut g, p, AVENGERS, &[]);
        // Bounded creature-layer fixture: the new enchantment has no printed animation ability.
        super::msh_galactus_tests::animate(&mut g, av, false);
        assert!(g.is_creature(av));
        paid(&mut g, p, "Frozen in Ice", &[Target::Card(av)]);
        assert!(!g.printed_abilities_available(av));
        assert_eq!(g.stats(hero).toughness, 4);
        g.move_object(hero, Zone::Graveyard);
        assert!(g.observe(p).turn_facts.is_some());
        g.begin_step(Step::End);
        assert!(!queued(&g, av));
    }
}
#[test]
fn msh_turn_facts_simultaneous_control_aura_captures_final_entry_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let hero = named(&mut g, p.other(), "Kid Loki", Zone::Hand);
        let aura = named(&mut g, p, "Confiscate", Zone::Hand);
        let mut control = g.entry_move(aura, Zone::Battlefield, false);
        control.attachment = Some(ObjectRef {
            card: hero,
            generation: g.object_ref(hero).generation + 1,
        });
        g.stage_entries(
            vec![g.entry_move(hero, Zone::Battlefield, false), control],
            None,
        );
        settle(&mut g);
        assert_eq!(g.object_view(hero).controller, p);
        assert!(facts(&g, p).hero_entered);
        assert!(!facts(&g, p.other()).hero_entered);
    }
}

#[test]
fn msh_turn_facts_actual_artifact_spell_only_funding_accepts_iron_rejects_avengers_atomically() {
    for p in [Player::P0, Player::P1] {
        for artifact in [true, false] {
            let mut g = setup(p);
            let castle = named(&mut g, p, "Castle Doom", Zone::Battlefield);
            g.state.players[p.index()].mana = [0; 6];
            checked(
                &mut g,
                Action::ActivateMana {
                    card: castle,
                    ability: 1,
                    payment: [0; 6],
                    counter_payment: Counters::default(),
                    production: [0, 0, 0, 0, 1, 0],
                },
            );
            g.state.players[p.index()].mana = if artifact {
                [0, 1, 0, 1, 1, 1]
            } else {
                [1, 0, 0, 0, 1, 3]
            };
            let id = named(
                &mut g,
                p,
                if artifact { IRON } else { AVENGERS },
                Zone::Hand,
            );
            if artifact {
                checked(&mut g, Action::AnnounceCast { card: id });
                super::msh_galactus_tests::finish(&mut g);
                settle(&mut g);
                assert!(g.on_battlefield(id));
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            } else {
                reject(&mut g, Action::AnnounceCast { card: id });
            }
        }
    }
}
