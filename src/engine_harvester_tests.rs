//! Any-origin current-incarnation history and whole Nightmare-copy resolution.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::*;
use crate::CreatureType;
fn gated(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    super::tests::spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn nightmare(g: &mut Game, p: Player) -> CardId {
    g.stage_copy_creation(
        p,
        CopyRecipe {
            definition: Card::from_name("Bear Cub").unwrap(),
            exceptions: CopyExceptions::NIGHTMARE,
        },
        1,
    );
    settle(g);
    g.state
        .copy_recipes
        .as_ref()
        .unwrap()
        .iter()
        .rev()
        .find(|r| {
            g.on_battlefield(r.object.card)
                && g.state.objects[r.object.card as usize].controller == p
        })
        .unwrap()
        .object
        .card
}
fn activate(g: &mut Game, source: CardId, target: CardId) {
    checked(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    while g.state.announcing.is_some() {
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(g, action);
    }
    assert!(g.state.objects[source as usize].tapped);
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
#[test]
fn graveyard_history_tracks_all_origins_exact_incarnations_and_actual_replacement_destinations() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let origins = [Zone::Hand, Zone::Library, Zone::Battlefield, Zone::Exile];
        let mut cards = Vec::new();
        for (i, origin) in origins.into_iter().enumerate() {
            let owner = if i % 2 == 0 { p } else { p.other() };
            let id = named(&mut g, owner, "Bear Cub", origin);
            g.move_object(id, Zone::Graveyard);
            cards.push(id);
            assert!(g.graveyard_entry_this_turn(g.object_ref(id)));
        }
        let discarded = named(&mut g, p.other(), "Llanowar Elves", Zone::Hand);
        g.discard_batch(&[discarded], None);
        assert!(g.graveyard_entry_this_turn(g.object_ref(discarded)));
        let countered = named(&mut g, p, "Serra Angel", Zone::Hand);
        cast(&mut g, countered, None, &[]);
        let stack = g.state.stack.last().unwrap().id;
        assert!(g.counter_stack_item(stack, false));
        assert!(g.graveyard_entry_this_turn(g.object_ref(countered)));
        let old = g.object_ref(cards[0]);
        g.move_object(cards[0], Zone::Hand);
        assert!(!g.graveyard_entry_this_turn(old));
        g.move_object(cards[0], Zone::Graveyard);
        assert_ne!(g.object_ref(cards[0]), old);
        assert!(g.graveyard_entry_this_turn(g.object_ref(cards[0])));
        let progenitus = gated(&mut g, p, "Progenitus", Zone::Hand);
        g.move_object(progenitus, Zone::Graveyard);
        assert_eq!(g.state.objects[progenitus as usize].zone, Zone::Library);
        assert!(!g.graveyard_entry_this_turn(g.object_ref(progenitus)));
        let token = nightmare(&mut g, p);
        g.move_object(token, Zone::Graveyard);
        assert!(!g.graveyard_entry_this_turn(g.object_ref(token)));
        assert_eq!(
            g.observe(p).graveyard_entries_this_turn,
            g.observe(p.other()).graveyard_entries_this_turn
        );
        g.begin_turn();
        assert!(g.state.graveyard_entries.is_none());
        assert!(!g.graveyard_entry_this_turn(g.object_ref(discarded)));
    }
}
#[test]
fn harvester_keeps_every_new_multiplied_copy_and_culls_other_current_friendly_nightmares() {
    for p in [Player::P0, Player::P1] {
        for doublers in 0..=2 {
            let mut g = setup(p);
            let old = nightmare(&mut g, p);
            let enemy = nightmare(&mut g, p.other());
            let physical = named(&mut g, p, "Dreadwing Scavenger", Zone::Battlefield);
            for _ in 0..doublers {
                named(&mut g, p, "Doubling Season", Zone::Battlefield);
            }
            let target = named(&mut g, p.other(), "Bear Cub", Zone::Hand);
            g.move_object(target, Zone::Graveyard);
            // History predates the source's entry and belongs to the game, not a watcher on it.
            let source = gated(&mut g, p, "Abyssal Harvester", Zone::Battlefield);
            activate(&mut g, source, target);
            g.move_object(source, Zone::Graveyard);
            settle(&mut g);
            assert_eq!(g.state.objects[target as usize].zone, Zone::Exile);
            assert_eq!(g.state.objects[old as usize].zone, Zone::Outside);
            assert!(g.on_battlefield(enemy));
            assert!(g.on_battlefield(physical));
            let copies: Vec<_> = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter(|(id, o)| {
                    o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.is_token()
                        && g.has_subtype(*id as CardId, "Nightmare")
                })
                .collect();
            assert_eq!(copies.len(), 1 << doublers);
            assert!(copies.iter().all(|(_, o)| o.card.name() == "Bear Cub"));
            assert!(g.state.nightmare_cull.is_none());
        }
    }
}
#[test]
fn harvester_illegal_or_new_incarnation_target_does_not_copy_or_cull_and_old_graves_are_not_targets()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let old_grave = named(&mut g, p, "Serra Angel", Zone::Graveyard);
        let target = named(&mut g, p, "Bear Cub", Zone::Hand);
        g.move_object(target, Zone::Graveyard);
        let old = nightmare(&mut g, p);
        let source = gated(&mut g, p, "Abyssal Harvester", Zone::Battlefield);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(old_grave),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        while g.state.announcing.is_some() {
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, action);
        }
        let bound = g.object_ref(target);
        g.move_object(target, Zone::Hand);
        g.move_object(target, Zone::Graveyard);
        assert_ne!(g.object_ref(target), bound);
        settle(&mut g);
        assert!(g.on_battlefield(old));
        assert_eq!(g.state.objects[target as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.copy_recipes.as_ref().unwrap().len(), 1);
        assert!(g.state.nightmare_cull.is_none());
    }
}
#[test]
fn harvester_waits_for_actual_copy_entry_choices_and_uses_current_type_and_controller_at_cull() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let old = nightmare(&mut g, p);
        let saved_by_type = nightmare(&mut g, p);
        let witness = named(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, witness, None, &[Target::Card(saved_by_type)]);
        settle(&mut g);
        let saved_by_control = nightmare(&mut g, p);
        let transfer = g.activation_item(p, saved_by_control, 0);
        g.gain_control(&transfer, Subject::Source, PlayerSet::Opponent);
        let becoming_friendly = nightmare(&mut g, p.other());
        let target = named(&mut g, p, "Adaptive Automaton", Zone::Hand);
        g.move_object(target, Zone::Graveyard);
        let source = gated(&mut g, p, "Abyssal Harvester", Zone::Battlefield);
        activate(&mut g, source, target);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert!(matches!(g.state.pending, Pending::EntryValue { .. }));
        let entering = g
            .state
            .copy_recipes
            .as_ref()
            .unwrap()
            .iter()
            .find(|r| r.recipe.definition.name() == "Adaptive Automaton")
            .unwrap()
            .object
            .card;
        assert_eq!(g.state.objects[entering as usize].zone, Zone::Outside);
        assert!(g.object_view(entering).has_subtype("Nightmare"));
        assert!(g.on_battlefield(old));
        let transfer = g.activation_item(p.other(), becoming_friendly, 0);
        g.gain_control(&transfer, Subject::Source, PlayerSet::Opponent);
        let value = EntryChoiceValue::CreatureType(CreatureType::from_name("Bear").unwrap());
        checked(&mut g, Action::ChooseEntryValue { value });
        settle(&mut g);
        assert_eq!(g.state.objects[old as usize].zone, Zone::Outside);
        assert_eq!(
            g.state.objects[becoming_friendly as usize].zone,
            Zone::Outside
        );
        assert!(g.on_battlefield(saved_by_control));
        assert!(g.on_battlefield(saved_by_type));
        assert!(!g.has_subtype(saved_by_type, "Nightmare"));
        let new = g
            .state
            .copy_recipes
            .as_ref()
            .unwrap()
            .iter()
            .find(|r| r.recipe.definition.name() == "Adaptive Automaton")
            .unwrap()
            .object
            .card;
        assert!(g.on_battlefield(new));
        assert!(g.has_subtype(new, "Nightmare"));
        assert!(g.state.nightmare_cull.is_none());
    }
}
