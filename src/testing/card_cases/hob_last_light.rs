//! Focused printed outcomes; shared engine invariants live in core tests.
//! Whole printed quest/search/cycling outcomes; hidden-zone choices remain owned.
use super::*;
const FIXTURES: &[&str] = &[
    "Last Light of Durin's Day",
    "Mountain",
    "Forest",
    "Shivan Dragon",
    "Barkform Harvester",
    "Doubling Season",
    "Rest in Peace",
    "Shock",
    "Bear Cub",
    "Secluded Courtyard",
    "Prophetic Prism",
];
fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    assert!(FIXTURES.contains(&name), "unlisted fixture {name}");
    put(g, p, name, zone)
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.resolving.is_none()
        && g.state.ordering_triggers.is_empty()
}
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..48 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("last-light action budget: {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    until(g, quiet);
}
fn search(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::HandLibrarySearch { .. })
    });
}
fn paid(g: &mut Game, p: Player) -> CardId {
    let id = fixture(g, p, "Last Light of Durin's Day", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
    cast(g, id, None);
    finish(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(g.object_view(id).zone, Zone::Battlefield);
    id
}
fn land(g: &mut Game, owner: Player, controller: Player, name: &str) -> CardId {
    let id = fixture(g, owner, name, Zone::Hand);
    let mut movement = g.entry_move(id, Zone::Battlefield, false);
    movement.controller = controller;
    g.stage_entries(vec![movement], None);
    g.prepare_triggers();
    id
}
fn ready(p: Player) -> (Game, CardId) {
    let mut g = scenario(p);
    let source = paid(&mut g, p);
    g.add_counter(source, CounterKind::Quest, 5);
    land(&mut g, p, p, "Mountain");
    search(&mut g);
    (g, source)
}
#[test]
fn last_light_paid_quest_trigger_uses_actual_subtype_control_and_resolution_threshold() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = paid(&mut g, p);
        land(&mut g, p, p, "Forest");
        finish(&mut g);
        land(&mut g, p, p.other(), "Mountain");
        finish(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::Quest), 0);
        land(&mut g, p.other(), p, "Mountain");
        assert_eq!(g.counter_count(source, CounterKind::Quest), 0); // event does not put counter
        finish(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::Quest), 1);
        g.add_counter(source, CounterKind::Quest, 4);
        land(&mut g, p, p, "Mountain");
        g.pay_counters(source, &Counters::of(CounterKind::Quest, 1));
        finish(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::Quest), 5);
        land(&mut g, p, p, "Mountain");
        search(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        act(
            &mut g,
            Action::ChooseHandLibrarySearchZones {
                hand: true,
                library: false,
            },
        );
        act(&mut g, Action::FinishSearch);
        finish(&mut g);
    }
}
#[test]
fn last_light_hidden_zone_modes_find_zero_or_dragon_and_shuffle_only_searched_library() {
    {
        let p = Player::P0;
        for (hand, library) in [(true, false), (false, true), (true, true)] {
            for find in [false, true] {
                let (mut g, source) = ready(p);
                let h = fixture(&mut g, p, "Shivan Dragon", Zone::Hand);
                let l = fixture(&mut g, p, "Shivan Dragon", Zone::Library);
                let other = fixture(&mut g, p.other(), "Shivan Dragon", Zone::Hand);
                let bear = fixture(&mut g, p, "Bear Cub", Zone::Hand);
                let old_rng = format!("{:?}", g.state.rng);
                let old_library = g.state.players[p.index()].library.clone();
                act(
                    &mut g,
                    Action::ChooseHandLibrarySearchZones { hand, library },
                );
                reject(&mut g, Action::ChooseSearch { card: other });
                reject(&mut g, Action::ChooseSearch { card: bear });
                assert_eq!(
                    g.legal_actions()
                        .contains(&Action::ChooseSearch { card: h }),
                    hand
                );
                assert_eq!(
                    g.legal_actions()
                        .contains(&Action::ChooseSearch { card: l }),
                    library
                );
                assert!(
                    g.observe(p.other())
                        .hand_library_search
                        .as_ref()
                        .unwrap()
                        .cards
                        .is_none()
                );
                assert!(
                    g.observe(p.other())
                        .objects
                        .iter()
                        .all(|o| o.id != h && o.id != l)
                );
                let chosen = if hand { h } else { l };
                if find {
                    act(&mut g, Action::ChooseSearch { card: chosen });
                }
                act(&mut g, Action::FinishSearch);
                finish(&mut g);
                assert_eq!(g.object_view(source).zone, Zone::Graveyard);
                assert_eq!(
                    g.object_view(chosen).zone,
                    if find {
                        Zone::Battlefield
                    } else if hand {
                        Zone::Hand
                    } else {
                        Zone::Library
                    }
                );
                assert_eq!(old_rng == format!("{:?}", g.state.rng), !library);
                if !library {
                    assert_eq!(old_library, g.state.players[p.index()].library);
                }
            }
        }
    }
}
#[test]
fn last_light_changeling_is_dragon_card_and_search_is_not_cast_or_creature_only() {
    let (mut g, _) = ready(Player::P0);
    let changeling = fixture(&mut g, Player::P0, "Barkform Harvester", Zone::Library);
    let program = Card::from_name("Last Light of Durin's Day")
        .unwrap()
        .triggers()[0]
        .program;
    assert!(matches!(
        program.last(),
        Some(Instruction::SearchHandAndOrLibrary(
            crate::SearchFilter::Subtype("Dragon")
        ))
    ));
    act(
        &mut g,
        Action::ChooseHandLibrarySearchZones {
            hand: false,
            library: true,
        },
    );
    act(&mut g, Action::ChooseSearch { card: changeling });
    act(&mut g, Action::FinishSearch);
    finish(&mut g);
    assert_eq!(g.object_view(changeling).zone, Zone::Battlefield);
    assert_eq!(stats(&g, changeling), (2, 3));
    assert_eq!(g.state.players[0].mana, [0; 6]);
}
