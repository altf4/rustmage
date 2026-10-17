use super::tests::{scenario, spawn};
use super::*;
const NAMES: [&str; 19] = [
    "Bilbo Baggins, Burglar // Take a Glance",
    "Frolicking Familiar // Blow Off Steam",
    "Galvanic Giant // Storm Reading",
    "Gingerbread Hunter // Puny Snack",
    "Glóin the Mighty // Easy Pickings",
    "Gollum, Silent Slinker // Meager Meal",
    "Grabby Giant // That's Mine",
    "Hollow Scavenger // Bakery Raid",
    "Imodane's Recruiter // Train Troops",
    "Midgar, City of Mako // Reactor Raid",
    "Minecart Daredevil // Ride the Rails",
    "Obyra's Attendants // Desperate Parry",
    "Ratcatcher Trainee // Pest Problem",
    "Shrouded Shepherd // Cleave Shadows",
    "Smaug, the Great Calamity // Spew Flame",
    "Stormkeld Vanguard // Bear Down",
    "Tempest Hart // Scan the Clouds",
    "Threadbind Clique // Rip the Seams",
    "Two-Headed Hunter // Twice the Rage",
];
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    g.state.players[p.index()].mana = [50; 6];
    for owner in [p, p.other()] {
        let bear = spawn(
            &mut g,
            owner,
            Card::from_name("Bear Cub").unwrap(),
            Zone::Battlefield,
        );
        g.state.objects[bear as usize].tapped = true;
        spawn(
            &mut g,
            owner,
            Card::from_name("Candy Trail").unwrap(),
            Zone::Battlefield,
        );
    }
    g
}
fn act(g: &mut Game, action: Action) {
    g.apply(g.actor().unwrap(), action).unwrap();
}
fn settle(g: &mut Game) {
    for _ in 0..150 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        let actions = g.legal_actions();
        let action = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .into_iter()
                .find(|a| !matches!(a, Action::Concede))
                .expect("continuation")
        };
        act(g, action);
    }
    panic!("unsettled {:?}", g.observe(g.actor().unwrap()));
}
#[test]
fn all_nineteen_whole_pairs_own_one_physical_card_and_both_cast_paths() {
    for p in [Player::P0, Player::P1] {
        for name in NAMES {
            for inset in [false, true] {
                let mut g = setup(p);
                let definition = Card::from_name(name).unwrap();
                assert!(definition.is_supported());
                let spell = definition.adventure_spell().unwrap();
                assert_eq!(spell.adventure_parent(), Some(definition));
                let id = spawn(&mut g, p, definition, Zone::Hand);
                let old = g.object_ref(id);
                let action = if inset {
                    Action::AnnounceAdventure { card: id }
                } else if definition.is_land() {
                    Action::PlayLand { card: id }
                } else {
                    Action::AnnounceCast { card: id }
                };
                assert!(g.legal_actions().contains(&action), "{name}, {inset}");
                act(&mut g, action);
                settle(&mut g);
                if inset {
                    assert_eq!(g.state.objects[id as usize].zone, Zone::Exile, "{name}");
                    assert_ne!(old, g.object_ref(id));
                    assert!(g.can_play_adventure(p, id));
                    assert!(
                        !g.legal_actions()
                            .contains(&Action::AnnounceAdventure { card: id })
                    );
                    g.state.priority = p;
                    let action = if definition.is_land() {
                        Action::PlayLand { card: id }
                    } else {
                        Action::AnnounceExile { card: id }
                    };
                    assert!(g.legal_actions().contains(&action), "later normal {name}");
                    act(&mut g, action);
                    settle(&mut g);
                }
                assert_eq!(
                    g.state.objects[id as usize].zone,
                    Zone::Battlefield,
                    "{name}"
                );
                assert_eq!(g.state.objects[id as usize].card, definition);
                assert!(!g.can_play_adventure(p, id));
            }
        }
    }
}
#[test]
fn chosen_instant_is_fundable_without_main_cost_and_has_only_inset_stack_traits() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.active = p.other();
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        let parent = Card::from_name("Frolicking Familiar // Blow Off Steam").unwrap();
        let id = spawn(&mut g, p, parent, Zone::Hand);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceAdventure { card: id })
        );
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: id })
        );
        act(&mut g, Action::AnnounceAdventure { card: id });
        let view = g.observe(p);
        let item = view.announcing.unwrap();
        assert_eq!(item.source_card, parent.adventure_spell().unwrap());
        assert!(item.source_card.is_instant());
        assert!(!item.source_card.is_creature());
        assert_eq!(item.mana_value(), 1);
        assert_eq!(item.source_card.colors(), 8);
        assert_eq!(item.cast.unwrap().physical_card, Some(parent));
        let v = g.object_view(id);
        assert_eq!(v.card, item.source_card);
        assert!(!v.is_creature());
        assert_eq!(v.keywords, Keywords::NONE);
        assert_eq!(v.power, 0);
    }
}
#[test]
fn internal_inset_definitions_are_rejected_even_from_freeform_decks() {
    let card = Card::from_name(NAMES[0])
        .unwrap()
        .adventure_spell()
        .unwrap();
    assert!(
        Game::new(Config {
            decks: [vec![card; 60], vec![Card::Forest; 60]],
            seed: 1,
            starting_player: Player::P0,
            deck_rules: DeckRules::Freeform
        })
        .is_err()
    );
}
fn announce_to_stack(g: &mut Game, action: Action, target: Option<Target>, id: CardId) {
    act(g, action);
    for _ in 0..50 {
        if g.state.pending == Pending::Priority
            && g.state
                .stack
                .iter()
                .any(|s| s.kind == StackKind::Spell && s.card == id)
        {
            return;
        }
        let actions = g.legal_actions();
        let chosen = target
            .and_then(|t| {
                actions
                    .iter()
                    .find(|a| **a == Action::ChooseTarget { target: t })
                    .cloned()
            })
            .or_else(|| actions.into_iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap();
        act(g, chosen);
    }
    panic!("announcement failed");
}
#[test]
fn countered_and_all_target_illegal_adventures_never_create_permissions() {
    for p in [Player::P0, Player::P1] {
        for counter in [true, false] {
            let mut g = setup(p);
            let q = p.other();
            g.state.players[q.index()].mana = [50; 6];
            let target = spawn(
                &mut g,
                q,
                Card::from_name("Bear Cub").unwrap(),
                Zone::Battlefield,
            );
            let parent = Card::from_name("Gingerbread Hunter // Puny Snack").unwrap();
            let id = spawn(&mut g, p, parent, Zone::Hand);
            announce_to_stack(
                &mut g,
                Action::AnnounceAdventure { card: id },
                Some(Target::Card(target)),
                id,
            );
            act(&mut g, Action::Pass);
            let response = spawn(
                &mut g,
                q,
                Card::from_name(if counter { "Counterspell" } else { "Shock" }).unwrap(),
                Zone::Hand,
            );
            announce_to_stack(
                &mut g,
                Action::AnnounceCast { card: response },
                Some(if counter {
                    Target::Spell(id)
                } else {
                    Target::Card(target)
                }),
                response,
            );
            settle(&mut g);
            assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
            assert!(!g.can_play_adventure(p, id));
            assert!(g.state.adventures.is_none());
        }
    }
}
#[test]
fn another_exile_permission_can_cast_inset_but_adventure_permission_alone_cannot() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let parent = Card::from_name("Minecart Daredevil // Ride the Rails").unwrap();
        let id = spawn(&mut g, p, parent, Zone::Library);
        g.exile_top_play(p, 1, true);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceAdventure { card: id })
        );
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceExile { card: id })
        );
        act(&mut g, Action::AnnounceAdventure { card: id });
        settle(&mut g);
        assert!(g.state.exile_permissions.is_empty());
        assert!(g.can_play_adventure(p, id));
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceAdventure { card: id })
        );
        let before = g.object_ref(id);
        let saved = g.snapshot();
        let mut child = {
            let parent = g.clone();
            let child = parent.clone();
            drop(parent);
            child
        };
        g.move_object(id, Zone::Hand);
        g.move_object(id, Zone::Exile);
        assert!(!g.can_play_adventure(p, id));
        assert_ne!(before, g.object_ref(id));
        g.restore(&saved).unwrap();
        assert!(g.can_play_adventure(p, id));
        act(&mut child, Action::AnnounceExile { card: id });
        settle(&mut child);
        assert_eq!(child.state.objects[id as usize].zone, Zone::Battlefield);
        assert!(g.can_play_adventure(p, id));
    }
}
#[test]
fn live_stack_damage_and_departed_lki_use_red_inset_not_blue_flying_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let parent = Card::from_name("Frolicking Familiar // Blow Off Steam").unwrap();
        let id = spawn(&mut g, p, parent, Zone::Hand);
        announce_to_stack(
            &mut g,
            Action::AnnounceAdventure { card: id },
            Some(Target::Player(p.other())),
            id,
        );
        let item = g.state.stack.last().unwrap().clone();
        for departed in [false, true] {
            if departed {
                g.move_object(id, Zone::Graveyard);
            }
            let source = g.program_damage_source(&item);
            assert_eq!(source.facts.card, parent.adventure_spell().unwrap());
            assert_eq!(source.facts.card.colors(), 8);
            assert_eq!(source.facts.kinds, 8);
            assert_eq!(source.keywords, Keywords::NONE);
            assert!(!source.facts.battlefield);
            assert_eq!(source.facts.power, 0);
            assert_eq!(source.reference, item.object);
        }
    }
}
#[test]
fn selected_face_reductions_and_typed_spell_predicates_ignore_main_creature_traits() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        spawn(
            &mut g,
            p,
            Card::from_name("Stormcatch Mentor").unwrap(),
            Zone::Battlefield,
        );
        let parent = Card::from_name("Minecart Daredevil // Ride the Rails").unwrap();
        let id = spawn(&mut g, p, parent, Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceAdventure { card: id })
        );
        announce_to_stack(&mut g, Action::AnnounceAdventure { card: id }, None, id);
        let item = g.state.stack.iter().find(|s| s.card == id).unwrap();
        assert_eq!(item.cast.unwrap().mana_spent(), 1);
        assert_eq!(item.mana_value(), 2);
        assert!(g.target_matches(
            TargetRule::SpellKinds {
                kinds: 12,
                excluded: 0
            },
            p.other(),
            g.object_ref(0),
            Target::Spell(id)
        ));
        assert!(!g.target_matches(
            TargetRule::SpellKinds {
                kinds: 1,
                excluded: 0
            },
            p.other(),
            g.object_ref(0),
            Target::Spell(id)
        ));
    }
}
#[test]
fn popped_spell_exit_captures_inset_characteristics_but_preserves_physical_card() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let parent = Card::from_name("Frolicking Familiar // Blow Off Steam").unwrap();
        let id = spawn(&mut g, p, parent, Zone::Hand);
        announce_to_stack(
            &mut g,
            Action::AnnounceAdventure { card: id },
            Some(Target::Player(p.other())),
            id,
        );
        let index = g.state.stack.iter().position(|s| s.card == id).unwrap();
        let item = g.state.stack.remove(index);
        let event = g.move_adventure_spell(&item, Zone::Graveyard);
        assert_eq!(event.before.card, parent.adventure_spell().unwrap());
        assert_eq!(event.before.kinds, 8);
        assert_eq!(event.before.keywords, Keywords::NONE);
        assert_eq!(event.before.power, 0);
        assert_eq!(g.state.objects[id as usize].card, parent);
        assert_eq!(g.object_view(id).card, parent);
        assert!(g.state.adventures.is_none());
    }
}

#[test]
fn late_stack_exit_and_later_normal_announcement_capacity_restore_exact_permissions() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let parent = Card::from_name("Minecart Daredevil // Ride the Rails").unwrap();
        let id = spawn(&mut g, p, parent, Zone::Hand);
        g.state.objects[id as usize].generation = u64::MAX - 1;
        announce_to_stack(&mut g, Action::AnnounceAdventure { card: id }, None, id);
        act(&mut g, Action::Pass);
        let saved = g.snapshot();
        assert!(matches!(
            g.apply(p.other(), Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), saved);
        assert!(g.state.adventures.is_none());

        let mut g = setup(p);
        let id = spawn(&mut g, p, parent, Zone::Hand);
        act(&mut g, Action::AnnounceAdventure { card: id });
        settle(&mut g);
        g.state.priority = p;
        g.state.next_stack_id = u64::MAX;
        let saved = g.snapshot();
        assert!(matches!(
            g.apply(p, Action::AnnounceExile { card: id }),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), saved);
        assert!(g.can_play_adventure(p, id));
    }
}

#[test]
fn normal_only_permission_survives_cleanup_but_retains_land_limits_and_actor() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let parent = Card::from_name("Midgar, City of Mako // Reactor Raid").unwrap();
        let id = spawn(&mut g, p, parent, Zone::Hand);
        act(&mut g, Action::AnnounceAdventure { card: id });
        settle(&mut g);
        let permission = g.state.adventures.as_ref().unwrap().permissions[0];
        // Indefinite Adventure grants do not use the cleanup deadline registry.
        g.state.step = Step::Cleanup;
        g.finish_cleanup();
        assert!(g.can_play_adventure(p, id));
        assert!(!g.can_play_adventure(p.other(), id));
        assert_eq!(
            g.state.adventures.as_ref().unwrap().permissions[0],
            permission
        );
        g.state.step = Step::Main1;
        g.state.active = p;
        g.state.priority = p;
        g.state.land_played = true;
        assert!(!g.legal_actions().contains(&Action::PlayLand { card: id }));
        g.state.land_played = false;
        let count = g.state.players[p.index()].spells_cast_this_turn;
        act(&mut g, Action::PlayLand { card: id });
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, count);
        assert!(g.state.objects[id as usize].tapped);
        assert!(g.state.adventures.is_none());
    }
}
