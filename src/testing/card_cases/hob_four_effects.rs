//! Focused printed outcomes; shared engine invariants live in core tests.
//! Paid Oracle-derived outcomes and representative contracts for four HOB spells.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..32 {
        if ready(g) {
            return;
        }
        if g.legal_actions().contains(&Action::Pass) {
            act(g, Action::Pass);
        } else {
            choose(g, |a| matches!(a, Action::PutTrigger { .. }));
        }
    }
    panic!("decision budget: {:?}", g.state.pending);
}
fn bind(g: &mut Game, target: Target) {
    act(g, Action::ChooseTarget { target });
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}
fn modal(g: &mut Game, id: CardId, mode: u16, target: Target) {
    act(g, Action::AnnounceCast { card: id });
    act(g, Action::ChooseMode { mode });
    bind(g, target);
}
fn library(g: &mut Game, p: Player, names: &[&str]) -> Vec<CardId> {
    let old = g.state.players[p.index()].library.to_vec();
    for id in old {
        g.state.objects[id as usize].zone = Zone::Outside;
    }
    g.state.players[p.index()].library.clear();
    let mut ids: Vec<_> = names
        .iter()
        .rev()
        .map(|name| put(g, p, name, Zone::Library))
        .collect();
    ids.reverse();
    ids
}
fn soldiers(g: &Game, p: Player) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Battlefield && o.controller == p && o.card.index() == 5241)
        .count()
}

#[test]
fn sound_uses_x_on_the_stack_and_does_nothing_for_a_vanished_target() {
    {
        let p = Player::P0;
        for x in [1, 2] {
            let mut g = scenario(p.other());
            let victim = put(&mut g, p.other(), "Wildwood Scourge", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: victim });
            act(&mut g, Action::ChooseX { value: x });
            pay(&mut g);
            act(&mut g, Action::Pass);
            let discard = put(&mut g, p, "Forest", Zone::Hand);
            let sound = put(&mut g, p, "Sound the Trumpets", Zone::Hand);
            cast(&mut g, sound, Some(Target::Spell(victim)));
            if x == 1 {
                until(&mut g, |g| {
                    matches!(g.state.pending, Pending::EffectDiscard { .. })
                });
                act(&mut g, Action::Discard { card: discard });
            }
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                u64::from(x == 1)
            );
            assert_eq!(soldiers(&g, p), 0);
        }
        let mut g = scenario(p.other());
        let victim = put(&mut g, p.other(), "Shock", Zone::Hand);
        cast(&mut g, victim, Some(Target::Player(p)));
        act(&mut g, Action::Pass);
        let sound = put(&mut g, p, "Sound the Trumpets", Zone::Hand);
        cast(&mut g, sound, Some(Target::Spell(victim)));
        let negate = put(&mut g, p, "Negate", Zone::Hand);
        cast(&mut g, negate, Some(Target::Spell(victim)));
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        assert_eq!(soldiers(&g, p), 0);
        assert_eq!(g.state.players[p.index()].life, 20);
    }
}
#[test]
fn gnashing_minus_five_registers_death_replacement_before_sbas_and_expires() {
    {
        let p = Player::P0;
        for survives in [false, true] {
            let mut g = scenario(p);
            let victim = put(
                &mut g,
                p.other(),
                if survives {
                    "Gigantosaurus"
                } else {
                    "Grizzly Bears"
                },
                Zone::Battlefield,
            );
            let spell = put(&mut g, p, "Gnashing of Teeth", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 1];
            modal(&mut g, spell, 0, Target::Card(victim));
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            if survives {
                assert_eq!(stats(&g, victim), (5, 5));
                let murder = put(&mut g, p, "Murder", Zone::Hand);
                g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 1];
                cast(&mut g, murder, Some(Target::Card(victim)));
                settle(&mut g);
            }
            assert_eq!(g.object_view(victim).zone, Zone::Exile);
        }
        let mut g = scenario(p);
        let victim = put(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        let spell = put(&mut g, p, "Gnashing of Teeth", Zone::Hand);
        modal(&mut g, spell, 0, Target::Card(victim));
        settle(&mut g);
        g.finish_cleanup();
        assert_eq!(stats(&g, victim), (10, 10));
        g.move_object(victim, Zone::Graveyard);
        assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
    }
}

#[test]
fn stone_destroy_rechecks_power_and_other_mode_preserves_characteristics_and_cleanup() {
    {
        let p = Player::P0;
        for lowered in [false, true] {
            let mut g = scenario(p);
            let small = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            let big = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            g.state.objects[big as usize].counters.plus_one = 1;
            let spell = put(&mut g, p, "Stone by Sunlight", Zone::Hand);
            g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
            act(&mut g, Action::AnnounceCast { card: spell });
            act(&mut g, Action::ChooseMode { mode: 0 });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(small),
                },
            );
            bind(&mut g, Target::Card(big));
            if lowered {
                g.state.objects[big as usize].counters.plus_one = 0;
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(big).zone,
                if lowered {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
        }
        let mut g = scenario(p);
        let victim = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.state.objects[victim as usize].counters.plus_one = 1;
        let before = g.object_view(victim);
        let spell = put(&mut g, p, "Stone by Sunlight", Zone::Hand);
        modal(&mut g, spell, 1, Target::Card(victim));
        settle(&mut g);
        let after = g.object_view(victim);
        assert_eq!(after.kinds, before.kinds | 16);
        assert_eq!(after.colors, before.colors);
        assert_eq!(stats(&g, victim), (3, 3));
        assert!(g.has_subtype(victim, "Bear"));
        assert!(g.keywords(victim).contains(Keywords::INDESTRUCTIBLE));
        let murder = put(&mut g, p, "Murder", Zone::Hand);
        cast(&mut g, murder, Some(Target::Card(victim)));
        settle(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Battlefield);

        g.finish_cleanup();
        assert_eq!(g.object_view(victim).kinds, 1);
        assert!(!g.keywords(victim).contains(Keywords::INDESTRUCTIBLE));
    }
}
#[test]
fn stone_adds_only_artifact_when_a_later_copy_removes_creature() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let shuri = put(&mut g, p, "Shuri, Wakandan Inventor", Zone::Battlefield);
        let victim = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let pick = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let spell = put(&mut g, p, "Stone by Sunlight", Zone::Hand);
        modal(&mut g, spell, 1, Target::Card(victim));
        settle(&mut g);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: shuri,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(pick),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(victim).kinds, 16);
        assert!(!g.is_creature(victim));
        assert!(g.keywords(victim).contains(Keywords::INDESTRUCTIBLE));
    }
}

#[test]
fn forest_gate_empty_library_no_lands_and_all_lands_are_valid_choices() {
    {
        let p = Player::P0;
        for names in [
            &[][..],
            &["Grizzly Bears", "Hill Giant"][..],
            &["Forest", "Mountain"][..],
        ] {
            let mut g = scenario(p);
            let top = library(&mut g, p, names);
            let spell = put(&mut g, p, "Through the Forest Gate", Zone::Hand);
            cast(&mut g, spell, None);
            if !top.is_empty() {
                until(&mut g, |g| {
                    g.legal_actions().contains(&Action::FinishLibrarySelection)
                });
                assert!(g.observe(p.other()).library_selection.is_none());
                for id in &top {
                    if g.state.objects[*id as usize].card.is_land() {
                        act(&mut g, Action::SelectLibrary { card: *id });
                    }
                }
                act(&mut g, Action::FinishLibrarySelection);
            }
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, 28);
            for id in top {
                if g.state.objects[id as usize].card.is_land() {
                    assert_eq!(g.object_view(id).zone, Zone::Battlefield);
                    assert!(g.object_view(id).tapped);
                } else {
                    assert_eq!(g.object_view(id).zone, Zone::Library);
                }
            }
        }
    }
}
#[test]
fn forest_gate_resumes_an_owned_entry_choice_before_shuffle_and_life_gain() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let top = library(
            &mut g,
            p,
            &[
                "Secluded Courtyard",
                "Forest",
                "Grizzly Bears",
                "Hill Giant",
            ],
        );
        let spell = put(&mut g, p, "Through the Forest Gate", Zone::Hand);
        cast(&mut g, spell, None);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::FinishLibrarySelection)
        });
        act(&mut g, Action::SelectLibrary { card: top[0] });
        act(&mut g, Action::SelectLibrary { card: top[1] });
        act(&mut g, Action::FinishLibrarySelection);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert!(g.observe(p).entry_value.is_some());
        assert!(g.observe(p.other()).library_selection.is_none());

        act(
            &mut g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::CreatureType(
                    crate::cards::CreatureType::from_name("Wizard").unwrap(),
                ),
            },
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 28);
        for id in &top[..2] {
            assert_eq!(g.object_view(*id).zone, Zone::Battlefield);
            assert!(g.object_view(*id).tapped);
        }
        assert_eq!(g.state.players[p.index()].library.len(), 2);
    }
}

#[test]
fn origin_of_metalbending_modes_and_target_restrictions() {
    {
        let p = Player::P0;
        for (name, mode) in [
            ("Murmuring Volume", 0),
            ("Phyrexian Arena", 0),
            ("Grizzly Bears", 1),
        ] {
            let mut g = scenario(p);
            let victim = put(&mut g, p, name, Zone::Battlefield);
            let opposing = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            let spell = put(&mut g, p, "Origin of Metalbending", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
            act(&mut g, Action::AnnounceCast { card: spell });
            act(&mut g, Action::ChooseMode { mode });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(opposing),
                },
            );
            bind(&mut g, Target::Card(victim));
            settle(&mut g);
            if mode == 0 {
                assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
            } else {
                assert_eq!(stats(&g, victim), (3, 3));
                assert!(g.keywords(victim).contains(Keywords::INDESTRUCTIBLE));
                g.finish_cleanup();
                assert!(!g.keywords(victim).contains(Keywords::INDESTRUCTIBLE));
                assert_eq!(stats(&g, victim), (3, 3));
            }
        }
    }
}
