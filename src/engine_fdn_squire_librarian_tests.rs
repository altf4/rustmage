use super::ability_timeline_tests::{cast, checked, named, next_turn, settle, setup};
use super::*;

fn play(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, targets);
    settle(g);
    id
}
fn main_phase(g: &mut Game) {
    for _ in 0..12 {
        if g.state.step == Step::Main1 {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("first main phase not reached");
}
fn mana(g: &mut Game) {
    for player in &mut g.state.players {
        player.mana = [30; 6];
    }
}
fn activate(g: &mut Game, id: CardId) {
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| {
            matches!(a,
                Action::Activate { card, ability: 0, target: None, .. } if *card == id
            )
        })
        .expect("funded Librarian activation");
    checked(g, action);
}
fn has_activation(g: &Game, id: CardId) -> bool {
    g.legal_actions().iter().any(|a| {
        matches!(a,
            Action::Activate { card, ability: 0, .. } if *card == id
        )
    })
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
fn assert_werewolf(g: &Game, id: CardId) {
    let object = g.object_view(id);
    assert_eq!(object.name(), "Mild-Mannered Librarian");
    assert!(object.has_subtype("Werewolf"));
    assert!(!object.has_subtype("Human"));
    assert_eq!(object.counters.plus_one, 2);
    assert_eq!(
        g.stats(id),
        Stats {
            power: 3,
            toughness: 3
        }
    );
    assert_eq!(object.exhausted_abilities, 1);
}

#[test]
fn fdn_squire_librarian_squire_paid_entries_count_tokens_other_creatures_and_only_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let squire = play(&mut g, p, "Skyknight Squire", &[]);
        assert_eq!(g.object_view(squire).counters.plus_one, 0);
        assert_eq!(g.object_view(squire).name(), "Skyknight Squire");
        play(&mut g, p, "Swiftfoot Boots", &[]);
        assert_eq!(g.object_view(squire).counters.plus_one, 0);
        play(&mut g, p, "Bear Cub", &[]);
        assert_eq!(g.object_view(squire).counters.plus_one, 1);
        assert!(!g.has_subtype(squire, "Knight"));
        assert!(!g.keywords(squire).contains(Keywords::FLYING));
        play(&mut g, p, "Resolute Reinforcements", &[]);
        assert_eq!(g.object_view(squire).counters.plus_one, 3);
        for kind in ["Cat", "Scout", "Knight"] {
            assert!(g.has_subtype(squire, kind));
        }
        assert!(g.keywords(squire).contains(Keywords::FLYING));
        assert_eq!(
            g.stats(squire),
            Stats {
                power: 4,
                toughness: 4
            }
        );
        next_turn(&mut g);
        main_phase(&mut g);
        assert_eq!(g.state.active, p.other());
        mana(&mut g);
        play(&mut g, p.other(), "Bear Cub", &[]);
        assert_eq!(g.object_view(squire).counters.plus_one, 3);
        // A real opposing aura removes the older layer-six flying grant, while
        // the same effect's layer-four Knight addition continues to apply.
        play(&mut g, p.other(), "Stop Cold", &[Target::Card(squire)]);
        assert!(g.has_subtype(squire, "Knight"));
        assert!(!g.keywords(squire).contains(Keywords::FLYING));
        next_turn(&mut g);
        main_phase(&mut g);
        mana(&mut g);
        play(&mut g, p, "Bear Cub", &[]);
        assert_eq!(g.object_view(squire).counters.plus_one, 3);
    }
}

#[test]
fn fdn_squire_librarian_squire_queued_trigger_does_not_grow_a_departed_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let squire = play(&mut g, p, "Skyknight Squire", &[]);
        let before = g.object_ref(squire);
        let creature = named(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, creature, None, &[]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert!(g.state.stack.iter().any(|item| item.object == before));
        let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
        cast(&mut g, bounce, None, &[Target::Card(squire)]);
        settle(&mut g);
        assert_eq!(g.object_view(squire).zone, Zone::Hand);
        assert_ne!(g.object_ref(squire), before);
        assert_eq!(g.object_view(squire).counters.plus_one, 0);
        cast(&mut g, squire, None, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(squire).counters.plus_one, 0);
        assert!(!g.has_subtype(squire, "Knight"));
    }
}

#[test]
fn fdn_squire_librarian_squire_entry_relation_tracks_its_current_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let squire = play(&mut g, p, "Skyknight Squire", &[]);
        let original = g.object_ref(squire);
        next_turn(&mut g);
        main_phase(&mut g);
        mana(&mut g);
        play(
            &mut g,
            p.other(),
            "Involuntary Employment",
            &[Target::Card(squire)],
        );
        assert_eq!(g.object_view(squire).controller, p.other());
        assert_eq!(g.object_ref(squire), original);
        // Flash permits the original owner to put real creatures onto the
        // battlefield during the new controller's turn; neither entry counts.
        checked(&mut g, Action::Pass);
        play(&mut g, p, "Resolute Reinforcements", &[]);
        assert_eq!(g.object_view(squire).counters.plus_one, 0);
        play(&mut g, p.other(), "Bear Cub", &[]);
        assert_eq!(g.object_view(squire).counters.plus_one, 1);
        next_turn(&mut g);
        main_phase(&mut g);
        assert_eq!(g.object_view(squire).controller, p);
        mana(&mut g);
        play(&mut g, p, "Bear Cub", &[]);
        assert_eq!(g.object_view(squire).counters.plus_one, 2);
    }
}

#[test]
fn fdn_squire_librarian_librarian_instant_once_effects_persist_and_draw_is_private() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let librarian = play(&mut g, p, "Mild-Mannered Librarian", &[]);
        assert!(g.object_view(librarian).has_subtype("Human"));
        next_turn(&mut g);
        mana(&mut g);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 4, 0];
        checked(&mut g, Action::Pass);
        assert_eq!(g.actor(), Some(p));
        let drawn = *g.state.players[p.index()].library.last().unwrap();
        let hand = g.hand_count(p);
        activate(&mut g, librarian);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(librarian).exhausted_abilities, 1);
        assert_eq!(g.object_view(librarian).counters.plus_one, 0);
        settle(&mut g);
        assert_werewolf(&g, librarian);
        assert_eq!(g.hand_count(p), hand + 1);
        assert_eq!(g.object_view(drawn).zone, Zone::Hand);
        assert!(g.observe(p).objects.iter().any(|object| object.id == drawn));
        assert!(
            !g.observe(p.other())
                .objects
                .iter()
                .any(|object| object.id == drawn)
        );
        next_turn(&mut g);
        mana(&mut g);
        assert_werewolf(&g, librarian);
        assert!(!has_activation(&g, librarian));
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: librarian,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::Activate {
                card: librarian,
                ability: 0,
                target: None,
                payment: [0, 0, 0, 0, 4, 0],
                counter_payment: Counters::default(),
            },
        );
    }
}

#[test]
fn fdn_squire_librarian_librarian_staged_payment_countered_limit_and_zone_reset() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let librarian = play(&mut g, p, "Mild-Mannered Librarian", &[]);
        let original = g.object_ref(librarian);
        for _ in 0..4 {
            named(&mut g, p, "Forest", Zone::Battlefield);
        }
        g.state.players[p.index()].mana = [0; 6];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: librarian,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        assert_eq!(g.object_view(librarian).exhausted_abilities, 0);
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        for _ in 0..4 {
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::TapMana { .. }))
                .unwrap();
            checked(&mut g, action);
        }
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 4, 0],
            },
        );
        assert_eq!(g.object_view(librarian).exhausted_abilities, 1);
        let hand = g.hand_count(p);
        let item = g.state.stack.last().unwrap().id;
        // The existing catalogue has no admitted ability-counter spell. Use its
        // actual shared counter routine rather than changing a printed card.
        assert!(g.counter_stack_item(item, false));
        assert_eq!(g.hand_count(p), hand);
        assert_eq!(g.object_view(librarian).counters.plus_one, 0);
        assert!(g.object_view(librarian).has_subtype("Human"));
        mana(&mut g);
        assert!(!has_activation(&g, librarian));
        play(&mut g, p, "Unsummon", &[Target::Card(librarian)]);
        assert_ne!(g.object_ref(librarian), original);
        assert_eq!(g.object_view(librarian).exhausted_abilities, 0);
        cast(&mut g, librarian, None, &[]);
        settle(&mut g);
        assert!(has_activation(&g, librarian));
        activate(&mut g, librarian);
        settle(&mut g);
        assert_werewolf(&g, librarian);
    }
}

#[test]
fn fdn_squire_librarian_librarian_departure_keeps_draw_but_not_old_source_changes() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let librarian = play(&mut g, p, "Mild-Mannered Librarian", &[]);
        let original = g.object_ref(librarian);
        activate(&mut g, librarian);
        let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
        cast(&mut g, bounce, None, &[Target::Card(librarian)]);
        let hand = g.hand_count(p);
        settle(&mut g);
        // Returning the creature adds one card and the independent draw adds one.
        assert_eq!(g.hand_count(p), hand + 2);
        assert_ne!(g.object_ref(librarian), original);
        assert_eq!(g.object_view(librarian).zone, Zone::Hand);
        assert_eq!(g.object_view(librarian).counters.plus_one, 0);
        assert_eq!(g.object_view(librarian).exhausted_abilities, 0);
        cast(&mut g, librarian, None, &[]);
        settle(&mut g);
        assert_eq!(
            g.stats(librarian),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert!(g.object_view(librarian).has_subtype("Human"));
        assert!(!g.object_view(librarian).has_subtype("Werewolf"));
        assert!(has_activation(&g, librarian));
    }
}

#[test]
fn fdn_squire_librarian_librarian_limit_and_effects_follow_controller_not_owner() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let librarian = play(&mut g, p, "Mild-Mannered Librarian", &[]);
        let original = g.object_ref(librarian);
        next_turn(&mut g);
        main_phase(&mut g);
        mana(&mut g);
        play(
            &mut g,
            p.other(),
            "Involuntary Employment",
            &[Target::Card(librarian)],
        );
        let hand = g.hand_count(p.other());
        let owner_hand = g.hand_count(p);
        activate(&mut g, librarian);
        settle(&mut g);
        assert_werewolf(&g, librarian);
        assert_eq!(g.hand_count(p.other()), hand + 1);
        assert_eq!(g.hand_count(p), owner_hand);
        assert_eq!(g.object_ref(librarian), original);
        next_turn(&mut g);
        main_phase(&mut g);
        mana(&mut g);
        assert_eq!(g.object_view(librarian).controller, p);
        assert_werewolf(&g, librarian);
        assert!(!has_activation(&g, librarian));
    }
}
