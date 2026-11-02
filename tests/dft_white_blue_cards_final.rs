//! Complete outcomes for the final W/U control, combat, copying and choice family.
mod support;
use rustmage as _;
use rustmage_core::CardName;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;
fn idle(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.resolving.is_none()
}
fn drain(g: &mut Game) {
    for _ in 0..80 {
        if idle(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::FinishTargets
                    | Action::FinishStackTargets
            )
        });
    }
    panic!("final family resolution ceiling");
}
fn ready(g: &mut Game, predicate: impl Fn(&Action) -> bool) {
    for _ in 0..40 {
        if g.legal_actions().iter().any(&predicate) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishStackTargets
            )
        });
    }
    panic!("final family decision ceiling");
}
fn finish(g: &mut Game) {
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn begin(g: &mut Game, id: CardId, ability: u16) {
    act(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
}
fn crew(g: &mut Game, id: CardId, contributors: &[CardId]) {
    begin(g, id, 0);
    if matches!(g.state.pending, Pending::PayCosts { .. }) {
        pay(g);
    }
    for &id in contributors {
        let object = g.object_ref(id);
        act(g, Action::ChooseCrewer { object });
    }
    act(g, Action::FinishCrew);
    drain(g);
}
fn attack(g: &mut Game, ids: &[CardId]) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    g.state.priority = g.state.active;
    g.state.passes = 0;
    for &card in ids {
        act(g, Action::Attack { card });
    }
    act(g, Action::FinishAttackers);
}
fn tokens(g: &Game, p: Player, subtype: &str) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            let id = i as CardId;
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(id, subtype))
            .then_some(id)
        })
        .collect()
}

#[test]
fn mu_yanling_creates_the_real_vehicle_grants_own_vehicles_flying_and_draws_once_for_a_combat_batch()
 {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Mu Yanling, Wind Rider", Zone::Hand);
    cast(&mut g, source, None);
    drain(&mut g);
    assert_eq!(stats(&g, source), (2, 4));
    let made = tokens(&g, p, "Vehicle");
    assert_eq!(made.len(), 1);
    let vehicle = made[0];
    assert!(!g.is_creature(vehicle));
    assert_eq!(g.current_kinds(vehicle), 16);
    assert_eq!(g.object_view(vehicle).colors, 0);
    assert!(g.keywords(vehicle).contains(Keywords::FLYING));
    let enemy = put(&mut g, p.other(), "Clamorous Ironclad", Zone::Battlefield);
    assert!(!g.keywords(enemy).contains(Keywords::FLYING));
    crew(&mut g, vehicle, &[source]);
    assert_eq!(stats(&g, vehicle), (3, 2));
    g.state.objects[vehicle as usize].summoning_sick = false;
    let flyer = put(&mut g, p, "Glitch Ghost Surveyor", Zone::Battlefield);
    attack(&mut g, &[vehicle, flyer]);
    for _ in 0..64 {
        if g.state.players[1].life == 15
            && g.state.players[0].cards_drawn_this_turn == 1
            && idle(&g)
        {
            break;
        }
        choose(&mut g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::FinishBlockers
                    | Action::PutTrigger { .. }
                    | Action::AssignDamage {
                        target: Target::Player(_),
                        ..
                    }
            )
        });
    }
    assert_eq!(g.state.players[1].life, 15);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    g.move_object(source, Zone::Graveyard);
    assert!(!g.keywords(vehicle).contains(Keywords::FLYING));
}

#[test]
fn possession_engine_control_and_combat_lock_end_irreversibly_when_the_vehicle_changes_controller()
{
    let p = Player::P0;
    let mut g = scenario(p);
    let target = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let source = put(&mut g, p, "Possession Engine", Zone::Hand);
    cast(&mut g, source, None);
    ready(&mut g, |a| {
        *a == Action::ChooseTarget {
            target: Target::Card(target),
        }
    });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    finish(&mut g);
    drain(&mut g);
    assert_eq!(g.object_view(target).controller, p);
    g.state.objects[target as usize].summoning_sick = false;
    assert!(!g.can_declare_attacker(p, target));
    g.state.active = p.other();
    assert!(!g.can_be_blocker(target));
    g.state.active = p;
    let pilot = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    crew(&mut g, source, &[pilot]);
    assert_eq!(stats(&g, source), (5, 5));
    let other = put(&mut g, p.other(), "Swiftfoot Boots", Zone::Battlefield);
    let exchange = put(&mut g, p, "Trade the Helm", Zone::Hand);
    announce(
        &mut g,
        exchange,
        &[Target::Card(source), Target::Card(other)],
    );
    drain(&mut g);
    assert_eq!(g.object_view(source).controller, p.other());
    assert_eq!(g.object_view(target).controller, p.other());
    let exchange = put(&mut g, p, "Trade the Helm", Zone::Hand);
    announce(
        &mut g,
        exchange,
        &[Target::Card(other), Target::Card(source)],
    );
    drain(&mut g);
    assert_eq!(g.object_view(source).controller, p);
    assert_eq!(g.object_view(target).controller, p.other());
    g.state.objects[target as usize].summoning_sick = false;
    g.state.objects[target as usize].tapped = false;
    assert!(g.can_declare_attacker(p.other(), target));
}

#[test]
fn sabotage_strategist_shrinks_only_the_declaration_attacking_its_player_and_exhausts_for_three_counters()
 {
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P1, "Sabotage Strategist", Zone::Battlefield);
    assert_eq!(stats(&g, source), (2, 2));
    assert!(
        g.keywords(source)
            .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
    );
    let a = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let b = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    let at_walker = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let walker = put(
        &mut g,
        Player::P1,
        "Ajani, Caller of the Pride",
        Zone::Battlefield,
    );
    g.add_counter(walker, CounterKind::Loyalty, 4);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    for card in [a, b] {
        act(&mut g, Action::Attack { card });
    }
    let defender = g.object_ref(walker);
    act(
        &mut g,
        Action::AttackPlaneswalker {
            card: at_walker,
            defender,
        },
    );
    act(&mut g, Action::FinishAttackers);
    let later = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    g.move_object(source, Zone::Graveyard);
    drain(&mut g);
    assert_eq!(stats(&g, a), (1, 2));
    assert_eq!(stats(&g, b), (2, 3));
    assert_eq!(stats(&g, at_walker), (2, 2));
    assert_eq!(stats(&g, later), (2, 2));
    let exhaust = put(&mut g, Player::P1, "Sabotage Strategist", Zone::Battlefield);
    act(&mut g, Action::Pass);
    g.state.players[1].mana = [0, 2, 0, 0, 0, 5];
    begin(&mut g, exhaust, 0);
    pay(&mut g);
    drain(&mut g);
    assert_eq!(stats(&g, exhaust), (5, 5));
    g.state.priority = Player::P1;
    g.state.players[1].mana = [8; 6];
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: exhaust,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    g.finish_cleanup();
    assert_eq!(stats(&g, a), (2, 2));
}

#[test]
fn salvation_engine_buffs_other_own_artifact_creatures_and_returns_an_artifact_on_attack() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Salvation Engine", Zone::Battlefield);
    let own = put(&mut g, p, "Guidelight Optimizer", Zone::Battlefield);
    let other = put(&mut g, p.other(), "Guidelight Optimizer", Zone::Battlefield);
    let normal = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    assert_eq!(stats(&g, own), (4, 3));
    assert_eq!(stats(&g, other), (2, 1));
    assert_eq!(stats(&g, normal), (2, 2));
    let a = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let b = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    crew(&mut g, source, &[a, b]);
    assert_eq!(stats(&g, source), (6, 10));
    let valid = put(&mut g, p, "Swiftfoot Boots", Zone::Graveyard);
    let old = g.object_ref(valid);
    let wrong = put(&mut g, p.other(), "Swiftfoot Boots", Zone::Graveyard);
    attack(&mut g, &[source]);
    ready(&mut g, |a| {
        *a == Action::ChooseTarget {
            target: Target::Card(valid),
        }
    });
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(wrong),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(valid),
        },
    );
    finish(&mut g);
    drain(&mut g);
    assert_eq!(g.object_view(valid).zone, Zone::Battlefield);
    assert_ne!(g.object_ref(valid), old);
    assert_eq!(g.object_view(wrong).zone, Zone::Graveyard);
}

#[test]
fn skyseers_chariot_names_a_nonland_and_taxes_both_players_actual_named_activations() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Skyseer's Chariot", Zone::Hand);
    cast(&mut g, source, None);
    ready(&mut g, |a| matches!(a, Action::ChooseEntryValue { .. }));
    reject(
        &mut g,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::CardName(CardName::from_name("Forest").unwrap()),
        },
    );
    act(
        &mut g,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::CardName(CardName::from_name("Aether Syphon").unwrap()),
        },
    );
    drain(&mut g);
    assert!(g.keywords(source).contains(Keywords::FLYING));
    let pilot = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    crew(&mut g, source, &[pilot]);
    assert_eq!(stats(&g, source), (3, 3));
    let own = put(&mut g, p, "Aether Syphon", Zone::Battlefield);
    let other = put(&mut g, p.other(), "Aether Syphon", Zone::Battlefield);
    g.state.players[0].mana = [0, 0, 0, 0, 0, 3];
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: own,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    g.state.players[0].mana = [0, 0, 0, 0, 0, 4];
    begin(&mut g, own, 0);
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    act(&mut g, Action::Pass);
    g.state.players[1].mana = [0, 0, 0, 0, 0, 4];
    begin(&mut g, other, 0);
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.state.players[1].mana, [0; 6]);
    assert_eq!(g.state.players[1].cards_drawn_this_turn, 1);
    g.move_object(source, Zone::Graveyard);
    g.state.priority = p;
    g.state.objects[own as usize].tapped = false;
    g.state.players[0].mana = [0, 0, 0, 0, 0, 2];
    begin(&mut g, own, 0);
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.state.players[0].mana, [0; 6]);
}

#[test]
fn slick_imitator_requires_max_speed_sacrifices_and_can_retarget_a_copy_of_your_spell() {
    let p = Player::P0;
    let mut g = scenario(p);
    let imitator = put(&mut g, p, "Slick Imitator", Zone::Hand);
    cast(&mut g, imitator, None);
    drain(&mut g);
    assert_eq!(stats(&g, imitator), (1, 3));
    assert_eq!(g.state.players[0].speed, 1);
    let bear = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let turtle = put(&mut g, p, "Aegis Turtle", Zone::Battlefield);
    let growth = put(&mut g, p, "Giant Growth", Zone::Hand);
    cast(&mut g, growth, Some(Target::Card(bear)));
    act(&mut g, Action::Pass);
    let shock = put(&mut g, p.other(), "Shock", Zone::Hand);
    cast(&mut g, shock, Some(Target::Player(p)));
    act(&mut g, Action::Pass);
    let action = Action::AnnounceSourceGrant {
        source: g.object_ref(imitator),
        ability: 0,
    };
    g.state.players[0].speed = 3;
    reject(&mut g, action.clone());
    g.state.players[0].speed = 4;
    g.state.players[0].mana = [0, 0, 0, 0, 0, 1];
    act(&mut g, action);
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Spell(shock),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Spell(growth),
        },
    );
    finish(&mut g);
    pay(&mut g);
    assert_eq!(g.object_view(imitator).zone, Zone::Graveyard);
    ready(&mut g, |a| matches!(a, Action::SetStackTarget { .. }));
    act(
        &mut g,
        Action::SetStackTarget {
            index: 0,
            target: Target::Card(turtle),
        },
    );
    act(&mut g, Action::FinishStackTargets);
    drain(&mut g);
    assert_eq!(stats(&g, bear), (5, 5));
    assert_eq!(stats(&g, turtle), (3, 8));
    assert_eq!(g.state.players[0].life, 18);
}

#[test]
fn slick_imitator_copy_of_a_permanent_spell_resolves_as_a_token_without_another_cast() {
    let p = Player::P0;
    let mut g = scenario(p);
    let imitator = put(&mut g, p, "Slick Imitator", Zone::Battlefield);
    g.state.players[0].speed = 4;
    let original = put(&mut g, p, "Grizzly Bears", Zone::Hand);
    cast(&mut g, original, None);
    let source = g.object_ref(imitator);
    act(&mut g, Action::AnnounceSourceGrant { source, ability: 0 });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Spell(original),
        },
    );
    finish(&mut g);
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.object_view(original).zone, Zone::Battlefield);
    let copies: Vec<_> = g
        .state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && o.card == Card::GrizzlyBears)
                .then_some(i as CardId)
        })
        .collect();
    assert_eq!(copies.len(), 1);
    assert_eq!(stats(&g, copies[0]), (2, 2));
    assert!(!g.state.objects[original as usize].is_token());
}

#[test]
fn spikeshell_bounces_the_opposing_vehicle_and_reduces_only_a_strict_leader_above_one() {
    for (own_speed, enemy_speed, expected) in
        [(2, 4, 3), (4, 4, 4), (3, 2, 2), (0, 1, 1), (1, 0, 0)]
    {
        let p = Player::P0;
        let mut g = scenario(p);
        g.state.players[0].speed = own_speed;
        g.state.players[1].speed = enemy_speed;
        let vehicle = put(&mut g, p, "Veloheart Bike", Zone::Battlefield);
        g.state.objects[vehicle as usize].controller = p.other();
        let own = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let source = put(&mut g, p, "Spikeshell Harrier", Zone::Hand);
        cast(&mut g, source, None);
        ready(&mut g, |a| {
            *a == Action::ChooseTarget {
                target: Target::Card(vehicle),
            }
        });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(vehicle),
            },
        );
        finish(&mut g);
        drain(&mut g);
        assert_eq!(g.object_view(vehicle).zone, Zone::Hand);
        assert_eq!(g.object_view(vehicle).owner, p);
        assert_eq!(g.state.players[1].speed, expected);
        assert_eq!(g.state.players[0].speed, own_speed);
        assert_eq!(stats(&g, source), (4, 4));
    }
}

#[test]
fn fabricator_creates_only_one_thopter_for_second_draw_and_resets_on_the_opponents_turn() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Thopter Fabricator", Zone::Hand);
    cast(&mut g, source, None);
    drain(&mut g);
    for _ in 0..2 {
        let draw = put(&mut g, p, "Divination", Zone::Hand);
        cast(&mut g, draw, None);
        drain(&mut g);
    }
    let made = tokens(&g, p, "Thopter");
    assert_eq!(made.len(), 1);
    let token = made[0];
    assert_eq!(stats(&g, token), (1, 1));
    assert_eq!(g.current_kinds(token), 17);
    assert_eq!(g.object_view(token).colors, 0);
    assert!(g.keywords(token).contains(Keywords::FLYING));
    let pilot = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    crew(&mut g, source, &[pilot]);
    assert_eq!(stats(&g, source), (4, 4));
    assert!(g.keywords(source).contains(Keywords::FLYING));
    g.finish_cleanup();
    assert_eq!(g.state.active, p.other());
    g.state.players[0].mana = [8; 6];
    act(&mut g, Action::Pass);
    let draw = put(&mut g, p, "Quick Study", Zone::Hand);
    cast(&mut g, draw, None);
    drain(&mut g);
    assert_eq!(tokens(&g, p, "Thopter").len(), 2);
}

#[test]
fn transit_mages_optional_search_accepts_only_four_or_five_mana_artifacts_and_decline_does_not_shuffle()
 {
    for selection in [None, Some("Alacrian Armory"), Some("Gilded Lotus")] {
        let p = Player::P0;
        let mut g = scenario(p);
        let four = put(&mut g, p, "Alacrian Armory", Zone::Library);
        let five = put(&mut g, p, "Gilded Lotus", Zone::Library);
        let wrong = put(&mut g, p, "Air Response Unit", Zone::Library);
        let not_artifact = put(&mut g, p, "Unswerving Sloth", Zone::Library);
        let before = g.state.players[0].library.clone();
        let source = put(&mut g, p, "Transit Mage", Zone::Hand);
        cast(&mut g, source, None);
        ready(&mut g, |a| matches!(a, Action::ChooseOptional { .. }));
        act(
            &mut g,
            Action::ChooseOptional {
                accept: selection.is_some(),
            },
        );
        if let Some(name) = selection {
            assert!(g.observe(p.other()).search_choice.is_none());
            for card in [wrong, not_artifact] {
                reject(&mut g, Action::ChooseSearch { card });
            }
            let card = if name == "Alacrian Armory" {
                four
            } else {
                five
            };
            act(&mut g, Action::ChooseSearch { card });
            act(&mut g, Action::FinishSearch);
            drain(&mut g);
            assert_eq!(g.object_view(card).zone, Zone::Hand);
        } else {
            drain(&mut g);
            assert_eq!(g.state.players[0].library, before);
        }
        assert_eq!(stats(&g, source), (2, 2));
    }
}

#[test]
fn valors_flagship_cycles_for_chosen_x_colorless_pilots_and_exactly_one_draw() {
    for x in [0, 2] {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Valor's Flagship", Zone::Hand);
        g.state.players[0].mana = [1, 0, 0, 0, 0, x + 2];
        begin(&mut g, source, 1);
        act(&mut g, Action::ChooseX { value: x });
        pay(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        drain(&mut g);
        let pilots = tokens(&g, p, "Pilot");
        assert_eq!(pilots.len(), x as usize);
        for &id in &pilots {
            assert_eq!(stats(&g, id), (1, 1));
            assert_eq!(g.object_view(id).colors, 0);
            assert_eq!(g.current_kinds(id), 1);
            assert!(g.keywords(id).contains(Keywords::CREW_SADDLE_PLUS_TWO));
        }
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
        assert_eq!(g.state.players[0].mana, [0; 6]);
        if let Some(&pilot) = pilots.first() {
            let ship = put(&mut g, p, "Valor's Flagship", Zone::Battlefield);
            crew(&mut g, ship, &[pilot]);
            assert_eq!(stats(&g, ship), (7, 7));
            assert!(
                g.keywords(ship).contains(
                    Keywords::FLYING
                        .union(Keywords::FIRST_STRIKE)
                        .union(Keywords::LIFELINK)
                )
            );
        }
    }
}

#[test]
fn valors_flagship_paid_entry_keeps_its_three_keywords_and_crews_for_three() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Valor's Flagship", Zone::Hand);
    let before: u32 = g.state.players[0].mana.iter().sum();
    cast(&mut g, source, None);
    drain(&mut g);
    assert_eq!(g.state.players[0].mana.iter().sum::<u32>(), before - 7);
    assert!(!g.is_creature(source));
    let pilot = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    crew(&mut g, source, &[pilot]);
    assert_eq!(stats(&g, source), (7, 7));
    assert!(
        g.keywords(source).contains(
            Keywords::FLYING
                .union(Keywords::FIRST_STRIKE)
                .union(Keywords::LIFELINK)
        )
    );
}

#[test]
fn waxen_flash_entry_copies_only_own_artifacts_or_creatures_with_owned_copy_choices_and_etbs() {
    let p = Player::P0;
    let mut parent = scenario(p.other());
    let creature = put(&mut parent, p, "Nimble Thopterist", Zone::Battlefield);
    let artifact = put(&mut parent, p, "Cultivator's Caravan", Zone::Battlefield);
    let enemy = put(
        &mut parent,
        p.other(),
        "Nimble Thopterist",
        Zone::Battlefield,
    );
    let wrong = put(&mut parent, p, "Unstoppable Plan", Zone::Battlefield);
    let source = put(&mut parent, p, "Waxen Shapethief", Zone::Hand);
    act(&mut parent, Action::Pass);
    cast(&mut parent, source, None);
    ready(&mut parent, |a| matches!(a, Action::ChooseEntryCopy { .. }));
    for id in [enemy, wrong] {
        let object = parent.object_ref(id);
        reject(
            &mut parent,
            Action::ChooseEntryCopy {
                object: Some(object),
            },
        );
    }
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let action = Action::ChooseEntryCopy {
        object: Some(parent.object_ref(creature)),
    };
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let actor = indexed.actor().unwrap();
    let revision = indexed.revision();
    act(&mut raw, action);
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    drop(parent);
    drain(&mut raw);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(stats(&raw, source), (3, 2));
    assert_eq!(tokens(&raw, p, "Thopter").len(), 1);
    assert!(!raw.keywords(source).contains(Keywords::FLASH));
    indexed.restore(&saved).unwrap();
    let object = indexed.object_ref(artifact);
    act(
        &mut indexed,
        Action::ChooseEntryCopy {
            object: Some(object),
        },
    );
    drain(&mut indexed);
    assert!(!indexed.is_creature(source));
    assert!(indexed.has_subtype(source, "Vehicle"));
    assert_eq!(tokens(&indexed, p, "Thopter").len(), 0);
    indexed.finish_cleanup();
    assert!(!indexed.is_creature(source));
    indexed.move_object(source, Zone::Hand);
    assert_eq!(
        indexed.state.objects[source as usize].card,
        Card::from_name("Waxen Shapethief").unwrap()
    );
    assert_eq!(stats(&raw, source), (3, 2));
    let mut decline = raw.clone();
    decline.restore(&saved).unwrap();
    act(&mut decline, Action::ChooseEntryCopy { object: None });
    drain(&mut decline);
    assert_eq!(decline.object_view(source).zone, Zone::Graveyard);
}

#[test]
fn waxen_cycling_pays_two_discards_the_original_and_draws_without_a_copy_choice() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Waxen Shapethief", Zone::Hand);
    g.state.players[0].mana = [0, 0, 0, 0, 0, 2];
    begin(&mut g, source, 0);
    pay(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    drain(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    assert!(g.observe(p).entry_copy.is_none());
}
