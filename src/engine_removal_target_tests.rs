//! Live legality for immutable removal predicates. Synthetic future-type objects
//! below exercise unions without admitting unsupported Vehicle/Spacecraft/walker cards.
use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
fn named(g: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, player, c, zone)
}
fn fund(g: &mut Game, id: CardId) {
    let (mut payment, generic) = g.state.objects[id as usize].card.cost();
    payment[5] += generic;
    g.state.players[0].mana = payment;
}
fn rule(name: &str) -> TargetRule {
    Card::from_name(name)
        .unwrap()
        .spell_program(None)
        .unwrap()
        .0
        .single()
        .unwrap()
        .rule
}
fn matches(g: &Game, name: &str, id: CardId) -> bool {
    g.target_matches(rule(name), Player::P0, g.object_ref(0), Target::Card(id))
}
fn replay(g: &mut Game, action: Action) {
    let saved = g.snapshot();
    let mut fork = g.clone();
    act(g, action.clone());
    act(&mut fork, action.clone());
    let expected = g.snapshot();
    assert_eq!(fork.snapshot(), expected);
    g.restore(&saved).unwrap();
    act(g, action);
    assert_eq!(g.snapshot(), expected);
}
#[test]
fn removal_predicates_use_current_stats_and_qualify_only_the_creature_union_branch() {
    let mut g = scenario();
    let bear = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let artifact = named(&mut g, Player::P1, "Mox Jasper", Zone::Battlefield);
    let enchantment = named(&mut g, Player::P1, "Phyrexian Arena", Zone::Battlefield);
    let land = named(&mut g, Player::P1, "Forest", Zone::Battlefield);
    assert!(!matches(&g, "Stand Up for Yourself", bear));
    g.state.objects[bear as usize].counters.plus_one = 1;
    assert!(matches(&g, "Stand Up for Yourself", bear));
    assert!(!matches(&g, "Repel Calamity", bear));
    g.state.objects[bear as usize].growth.toughness = 2;
    assert!(matches(&g, "Repel Calamity", bear));
    assert!(matches(&g, "Gallant Strike", bear));
    assert!(!matches(&g, "Make Your Move", bear));
    g.state.objects[bear as usize].growth.power = i128::from(u64::MAX);
    assert!(matches(&g, "Make Your Move", bear));
    for name in ["Broken Wings", "Make Your Move", "Exorcise"] {
        assert!(matches(&g, name, artifact), "{name}");
        assert!(matches(&g, name, enchantment), "{name}");
        assert!(!matches(&g, name, land), "{name}");
    }
    assert!(!matches(&g, "Broken Wings", bear));
    g.state.objects[bear as usize].gained_keywords = Keywords::FLYING;
    assert!(matches(&g, "Broken Wings", bear));
    g.move_object(bear, Zone::Graveyard);
    assert!(!matches(&g, "Broken Wings", bear));
}
#[test]
fn colors_include_hybrid_halves_and_token_colors_not_color_identity_or_payment() {
    let mut g = scenario();
    let hybrid = named(&mut g, Player::P1, "Cat-Owl", Zone::Battlefield);
    let green = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let colorless = named(&mut g, Player::P1, "Mox Jasper", Zone::Battlefield);
    assert_eq!(Card::from_name("Cat-Owl").unwrap().colors(), 3);
    assert!(matches(&g, "Deathmark", hybrid));
    assert!(matches(&g, "Refute Destiny", hybrid));
    assert!(matches(&g, "Terminal Criticism", hybrid));
    assert!(!matches(&g, "Devout Decree", hybrid));
    assert!(matches(&g, "Deathmark", green));
    for n in [
        "Deathmark",
        "Devout Decree",
        "Refute Destiny",
        "Terminal Criticism",
    ] {
        assert!(!matches(&g, n, colorless));
    }
    let knight = Card::catalogue()
        .find(|c| {
            c.token_info()
                .is_some_and(|t| t.subtypes == "Knight" && t.colors == 1)
        })
        .unwrap();
    let token = spawn(&mut g, Player::P1, knight, Zone::Battlefield);
    assert_eq!(knight.colors(), 1);
    assert!(matches(&g, "Deathmark", token));
    assert!(!matches(&g, "Refute Destiny", token));
    assert_eq!(Card::Forest.colors(), 0);
    assert_eq!(Card::GrizzlyBears.colors(), 16);
}
#[test]
fn outlaw_subtypes_include_changeling_and_never_vehicle_or_spacecraft() {
    let mut g = scenario();
    let bear = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let assassin = named(&mut g, Player::P1, "Rooftop Assassin", Zone::Battlefield);
    assert!(matches(&g, "Shoot the Sheriff", bear));
    assert!(!matches(&g, "Shoot the Sheriff", assassin));
    let changeling = Card::catalogue()
        .find(|c| {
            c.is_supported() && c.is_creature() && c.keywords().contains(Keywords::CHANGELING)
        })
        .unwrap();
    let id = spawn(&mut g, Player::P1, changeling, Zone::Battlefield);
    assert!(!matches(&g, "Shoot the Sheriff", id));
    for (subtype, spell) in [("Vehicle", "Spin Out"), ("Spacecraft", "Gravkill")] {
        let c = Card::catalogue()
            .find(|c| !c.is_creature() && c.has_subtype(subtype))
            .unwrap();
        let id = spawn(&mut g, Player::P1, c, Zone::Battlefield); // Synthetic unsupported future type.
        assert!(matches(&g, spell, id));
        assert!(!matches(&g, "Shoot the Sheriff", id));
        assert_eq!(matches(&g, "Gravkill", id), subtype == "Spacecraft");
        assert_eq!(matches(&g, "Spin Out", id), subtype == "Vehicle");
    }
}
#[test]
fn combat_filters_track_blocking_after_attacker_leaves_and_reject_tapped_bystanders() {
    let mut g = scenario();
    let attacker = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let blocker = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let bystander = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    g.state.objects[bystander as usize].tapped = true;
    for id in [attacker, blocker, bystander] {
        assert!(!matches(&g, "Sudden Strike", id));
    }
    g.state.attackers.push(Attacker {
        defender: Some(CombatDefender::Player(Player::P1)),
        card: attacker,
        blocked: true,
    });
    g.state.blocking.push(blocker);
    g.state.blocks.push((blocker, attacker));
    assert!(matches(&g, "Not on My Watch", attacker));
    assert!(!matches(&g, "Not on My Watch", blocker));
    assert!(matches(&g, "Sudden Strike", blocker));
    g.move_object(attacker, Zone::Hand);
    assert!(matches(&g, "Sudden Strike", blocker));
    assert!(!matches(&g, "Not on My Watch", attacker));
    assert!(!matches(&g, "Sudden Strike", bystander));
    assert!(matches(&g, "Radiant Strike", bystander));
    g.move_object(blocker, Zone::Hand);
    g.move_object(blocker, Zone::Battlefield);
    assert!(!matches(&g, "Sudden Strike", blocker));
}
#[test]
fn unions_preserve_hexproof_and_resolution_fizzles_suppress_the_whole_tail() {
    let mut g = scenario();
    let target = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Radiant Strike", Zone::Hand);
    g.state.objects[target as usize].tapped = true;
    fund(&mut g, spell);
    cast(&mut g, spell, Some(Target::Card(target)));
    g.state.objects[target as usize].tapped = false;
    let life = g.state.players[0].life;
    replay(&mut g, Action::Pass);
    replay(&mut g, Action::Pass);
    assert_eq!(g.state.players[0].life, life);
    assert_eq!(g.object_view(target).zone, Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Make Your Move", Zone::Hand);
    let artifact = named(&mut g, Player::P1, "Mox Jasper", Zone::Battlefield);
    g.state.objects[artifact as usize].gained_keywords = Keywords::HEXPROOF;
    assert!(!matches(&g, "Make Your Move", artifact));
    fund(&mut g, spell);
    let before = g.snapshot();
    let rev = g.revision();
    assert!(
        g.apply(
            Player::P0,
            Action::Cast {
                card: spell,
                target: Some(Target::Card(artifact)),
                payment: g.state.players[0].mana
            }
        )
        .is_err()
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
    g.state.objects[artifact as usize].gained_keywords = Keywords::NONE;
    cast(&mut g, spell, Some(Target::Card(artifact)));
    g.move_object(artifact, Zone::Hand);
    g.move_object(artifact, Zone::Battlefield);
    pass_pair(&mut g);
    assert_eq!(g.object_view(artifact).zone, Zone::Battlefield);
}
#[test]
fn printed_simple_removals_resolve_with_exact_tail_and_branch_replay() {
    let cases = [
        ("Not on My Watch", "Bear Cub", true, true, 0),
        ("Spin Out", "Bear Cub", false, false, 0),
        ("Gravkill", "Bear Cub", false, true, 0),
        ("Deathmark", "Bear Cub", false, false, 0),
        ("Mortify", "Bear Cub", false, false, 0),
        ("Hero's Downfall", "Bear Cub", false, false, 0),
        ("Sudden Strike", "Bear Cub", true, false, 0),
        ("Death in the Family", "Bear Cub", false, true, 0),
        ("Eriette's Lullaby", "Bear Cub", false, false, 2),
        ("Stand Up for Yourself", "Hill Giant", false, false, 0),
        ("Grapple with Death", "Bear Cub", false, false, 1),
        ("Withering Torment", "Bear Cub", false, false, -2),
        ("Radiant Strike", "Bear Cub", false, false, 3),
        ("Repel Calamity", "Hill Giant", false, false, 0),
        ("Broken Wings", "Cat-Owl", false, false, 0),
        ("Gallant Strike", "Hill Giant", false, false, 0),
        ("Exorcise", "Mox Jasper", false, true, 0),
        ("Make Your Move", "Mox Jasper", false, false, 0),
        ("Shoot the Sheriff", "Bear Cub", false, false, 0),
        ("Terminal Criticism", "Cat-Owl", false, false, 1),
    ];
    for (name, victim, combat, exile, life_delta) in cases {
        let mut g = scenario();
        let target = named(&mut g, Player::P1, victim, Zone::Battlefield);
        g.state.objects[target as usize].tapped = true;
        if combat {
            g.state.attackers.push(Attacker {
                defender: Some(CombatDefender::Player(Player::P1)),
                card: target,
                blocked: false,
            });
        }
        if victim == "Hill Giant" {
            g.state.objects[target as usize].counters.plus_one = 1;
        }
        let spell = named(&mut g, Player::P0, name, Zone::Hand);
        fund(&mut g, spell);
        cast(&mut g, spell, Some(Target::Card(target)));
        replay(&mut g, Action::Pass);
        replay(&mut g, Action::Pass);
        assert_eq!(
            g.object_view(target).zone,
            if exile { Zone::Exile } else { Zone::Graveyard },
            "{name}"
        );
        assert_eq!(g.state.players[0].life, 20 + life_delta, "{name}");
    }
}
fn pay(g: &mut Game) {
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .expect("payment");
    replay(g, action);
}
fn settle(g: &mut Game) {
    for _ in 0..500 {
        if g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.announcing.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        let a = actions[crate::policy::smoke_index(g, &actions)].clone();
        assert_ne!(a, Action::Concede);
        replay(g, a);
    }
    panic!("fixture didn't settle");
}
#[test]
fn every_new_modal_mode_executes_its_whole_printed_program_and_replays() {
    for name in [
        "Battle Menu",
        "Pawpatch Formation",
        "Sarkhan's Resolve",
        "Scout the City",
        "Unforgiving Aim",
        "Valorous Stance",
    ] {
        let card = Card::from_name(name).unwrap();
        let Effect::Modal(modes) = card.effect() else {
            panic!("modal {name}")
        };
        for mode in 0..modes.len() {
            let mut g = scenario();
            let own = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
            let big = named(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
            g.state.objects[big as usize].counters.plus_one = 1;
            let flyer = named(&mut g, Player::P1, "Cat-Owl", Zone::Battlefield);
            let enchantment = named(&mut g, Player::P1, "Phyrexian Arena", Zone::Battlefield);
            let spell = named(&mut g, Player::P0, name, Zone::Hand);
            fund(&mut g, spell);
            let drawn = g.state.players[0].cards_drawn_this_turn;
            let before_lib = g.state.players[0].library.len();
            replay(&mut g, Action::AnnounceCast { card: spell });
            replay(&mut g, Action::ChooseMode { mode: mode as u16 });
            let chosen = match (name, mode) {
                ("Battle Menu", 1) | ("Sarkhan's Resolve", 0) | ("Valorous Stance", 0) => Some(own),
                ("Battle Menu", 2) | ("Valorous Stance", 1) => Some(big),
                ("Pawpatch Formation" | "Unforgiving Aim", 1) => Some(enchantment),
                ("Battle Menu", 0 | 3)
                | ("Pawpatch Formation" | "Unforgiving Aim", 2)
                | ("Scout the City", 0) => None,
                _ => Some(flyer),
            };
            if let Some(id) = chosen {
                replay(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(id),
                    },
                );
            }
            pay(&mut g);
            settle(&mut g);
            match (name, mode) {
                ("Battle Menu", 0) | ("Unforgiving Aim", 2) => {
                    let token = g
                        .state
                        .objects
                        .iter()
                        .find(|o| o.zone == Zone::Battlefield && o.card.is_token())
                        .unwrap();
                    assert_eq!(
                        token.card.colors(),
                        if name == "Battle Menu" { 1 } else { 20 }
                    );
                    assert_eq!((token.local_power(), token.local_toughness()), (2, 2));
                }
                ("Battle Menu", 1) => {
                    assert_eq!((g.stats(own).power, g.stats(own).toughness), (2, 6))
                }
                ("Battle Menu", 3) => assert_eq!(g.state.players[0].life, 24),
                ("Pawpatch Formation", 2) => {
                    assert_eq!(g.state.players[0].cards_drawn_this_turn, drawn + 1);
                    assert!(
                        g.state
                            .objects
                            .iter()
                            .any(|o| o.zone == Zone::Battlefield && o.card.has_subtype("Food"))
                    );
                }
                ("Sarkhan's Resolve", 0) => {
                    assert_eq!((g.stats(own).power, g.stats(own).toughness), (5, 5))
                }
                ("Valorous Stance", 0) => {
                    assert!(g.keywords(own).contains(Keywords::INDESTRUCTIBLE))
                }
                ("Scout the City", 0) => {
                    assert_eq!(g.state.players[0].life, 23);
                    assert_eq!(g.state.players[0].library.len(), before_lib - 3);
                }
                _ => assert_eq!(
                    g.object_view(chosen.unwrap()).zone,
                    Zone::Graveyard,
                    "{name} mode{mode}"
                ),
            }
        }
    }
}
#[test]
fn new_removal_tails_and_mandatory_additional_costs_use_native_continuations() {
    for (name, victim, exile) in [
        ("Devout Decree", "Rooftop Assassin", true),
        ("Ray of Ruin", "Bear Cub", true),
        ("Refute Destiny", "Bear Cub", true),
        ("Shattered Wings", "Cat-Owl", false),
        ("Your Fate Ends Here", "Hill Giant", false),
    ] {
        let mut g = scenario();
        let target = named(&mut g, Player::P1, victim, Zone::Battlefield);
        let spell = named(&mut g, Player::P0, name, Zone::Hand);
        fund(&mut g, spell);
        cast(&mut g, spell, Some(Target::Card(target)));
        replay(&mut g, Action::Pass);
        replay(&mut g, Action::Pass);
        assert_eq!(
            g.object_view(target).zone,
            if exile { Zone::Exile } else { Zone::Graveyard }
        );
        assert!(
            g.state
                .resolving
                .as_ref()
                .is_some_and(|r| r.library.is_some()),
            "{name}"
        );
        settle(&mut g);
    }
    for (name, exile) in [("Embrace Oblivion", false), ("Worthy Cost", true)] {
        let mut g = scenario();
        let cost = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        let target = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, Player::P0, name, Zone::Hand);
        fund(&mut g, spell);
        replay(&mut g, Action::AnnounceCast { card: spell });
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        let object = g.object_ref(cost);
        replay(&mut g, Action::ChooseAdditionalCost { object });
        assert_eq!(g.object_view(cost).zone, Zone::Battlefield);
        pay(&mut g);
        assert_eq!(g.object_view(cost).zone, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(
            g.object_view(target).zone,
            if exile { Zone::Exile } else { Zone::Graveyard }
        );
    }
}
#[test]
fn new_entry_and_attack_removals_bind_opponent_tokens_damage_and_graveyard_generations() {
    for name in [
        "Kraul Whipcracker",
        "Meteor Golem",
        "Meteor Sword",
        "Rooftop Assassin",
        "Stingblade Assassin",
        "Disruptor Wanderglyph",
    ] {
        let mut g = scenario();
        let token = Card::catalogue()
            .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Knight"))
            .unwrap();
        let victim = if name == "Kraul Whipcracker" {
            spawn(&mut g, Player::P1, token, Zone::Battlefield)
        } else {
            named(
                &mut g,
                Player::P1,
                "Bear Cub",
                if name == "Disruptor Wanderglyph" {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                },
            )
        };
        let bystander = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        g.state.objects[victim as usize].damage = 1;
        g.state.objects[victim as usize].damaged_this_turn = true;
        let source = named(&mut g, Player::P0, name, Zone::Hand);
        g.move_object(source, Zone::Battlefield);
        if name == "Disruptor Wanderglyph" {
            g.state.objects[source as usize].summoning_sick = false;
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            replay(&mut g, Action::Attack { card: source });
            replay(&mut g, Action::FinishAttackers);
        } else {
            g.prepare_triggers();
        }
        assert!(
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(victim)
            }),
            "{name}"
        );
        if matches!(
            name,
            "Kraul Whipcracker"
                | "Rooftop Assassin"
                | "Stingblade Assassin"
                | "Disruptor Wanderglyph"
        ) {
            assert!(!g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(bystander)
            }));
        }
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        settle(&mut g);
        assert_eq!(
            g.object_view(victim).zone,
            if name == "Disruptor Wanderglyph" {
                Zone::Exile
            } else if name == "Kraul Whipcracker" {
                Zone::Outside
            } else {
                Zone::Graveyard
            },
            "{name}"
        );
        if name == "Meteor Sword" {
            let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
            g.state.players[0].mana = [0, 0, 0, 0, 0, 3];
            g.state.step = Step::Main1;
            g.give_priority();
            let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate {card,target:Some(Target::Card(t)),..}if *card==source&&*t==bear)).unwrap();
            replay(&mut g, a);
            settle(&mut g);
            assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (5, 5));
        }
    }
}
#[test]
fn new_sacrifice_activations_keep_timing_costs_and_removed_source_effects() {
    for (name, ability) in [
        ("Goblin Firebomb", 0),
        ("Bumbleflower's Sharepot", 0),
        ("Giant's Boulder", 1),
        ("Scrap Compactor", 0),
        ("Scrap Compactor", 1),
    ] {
        let mut g = scenario();
        let target = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        let source = named(&mut g, Player::P0, name, Zone::Battlefield);
        g.state.players[0].mana = [0, 0, 0, 0, 0, 10];
        let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate {card,ability:i,target:Some(Target::Card(t)),..}if *card==source&&*i==ability&&*t==target)).unwrap();
        replay(&mut g, a);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard, "{name}");
    }
    for name in ["Airship Crash", "Gallant Strike"] {
        let mut g = scenario();
        let spell = named(&mut g, Player::P0, name, Zone::Hand);
        g.state.players[0].mana = [0, 0, 0, 0, 0, 2];
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a,Action::Activate {card,..}if *card==spell))
            .unwrap();
        replay(&mut g, a);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    }
}
#[test]
fn boulder_and_sharepot_keep_their_other_entry_and_mana_abilities() {
    for name in ["Giant's Boulder", "Bumbleflower's Sharepot"] {
        let mut g = scenario();
        let source = named(&mut g, Player::P0, name, Zone::Hand);
        fund(&mut g, source);
        cast(&mut g, source, None);
        pass_pair(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        assert!(
            g.state
                .stack
                .iter()
                .any(|s| s.card == source && matches!(s.kind, StackKind::TriggeredAbility { .. }))
        );
        pass_pair(&mut g);
        if name == "Giant's Boulder" {
            assert!(
                g.state
                    .resolving
                    .as_ref()
                    .is_some_and(|r| r.library.is_some())
            );
            settle(&mut g);
            g.state.players[0].mana = [0, 0, 0, 0, 0, 1];
            let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana {card,production,..}if *card==source&&*production==[0,0,0,0,1,0])).unwrap();
            replay(&mut g, action);
            assert_eq!(g.state.players[0].mana, [0, 0, 0, 0, 1, 0]);
            assert!(g.object_view(source).tapped);
        } else {
            settle(&mut g);
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.zone == Zone::Battlefield && o.card.has_subtype("Food"))
                    .count(),
                1
            );
        }
    }
}
#[test]
fn power_and_keyword_restrictions_revalidate_after_response_and_destroy_respects_indestructible() {
    for (name, keyword) in [("Make Your Move", false), ("Broken Wings", true)] {
        let mut g = scenario();
        let target = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        if keyword {
            g.state.objects[target as usize].gained_keywords = Keywords::FLYING;
        } else {
            g.state.objects[target as usize].counters.plus_one = 2;
        }
        let spell = named(&mut g, Player::P0, name, Zone::Hand);
        fund(&mut g, spell);
        cast(&mut g, spell, Some(Target::Card(target)));
        // Synthetic response mutations exercise live revalidation independently of the predicate query.
        g.state.objects[target as usize].gained_keywords = Keywords::NONE;
        g.state.objects[target as usize].counters.plus_one = 0;
        replay(&mut g, Action::Pass);
        replay(&mut g, Action::Pass);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
    }
    let mut g = scenario();
    let target = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    g.state.objects[target as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
    let spell = named(&mut g, Player::P0, "Grapple with Death", Zone::Hand);
    fund(&mut g, spell);
    cast(&mut g, spell, Some(Target::Card(target)));
    settle(&mut g);
    assert_eq!(g.object_view(target).zone, Zone::Battlefield);
    assert_eq!(g.state.players[0].life, 21);
}
#[test]
fn dealt_damage_history_survives_cleanup_priority_and_resets_only_at_new_turn_or_incarnation() {
    let mut g = scenario();
    let victim = named(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    let shock = named(&mut g, Player::P0, "Shock", Zone::Hand);
    fund(&mut g, shock);
    cast(&mut g, shock, Some(Target::Card(victim)));
    settle(&mut g);
    assert_eq!(g.object_view(victim).damage, 2);
    assert!(g.object_view(victim).damaged_this_turn);
    let fragile = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    g.state.objects[fragile as usize].counters.minus_one = 2;
    g.state.objects[fragile as usize].growth.toughness = 1;
    let assassin = named(&mut g, Player::P0, "Rooftop Assassin", Zone::Hand);
    g.state.step = Step::End;
    g.state.passes = 1;
    let turn = g.state.turn;
    replay(&mut g, Action::Pass);
    assert_eq!(g.state.step, Step::Cleanup);
    assert_eq!(g.state.turn, turn);
    assert_eq!(g.object_view(victim).damage, 0);
    assert!(g.object_view(victim).damaged_this_turn);
    assert!(
        g.observe(Player::P1)
            .objects
            .iter()
            .find(|o| o.id == victim)
            .unwrap()
            .damaged_this_turn
    );
    let saved = g.snapshot();
    fund(&mut g, assassin);
    cast(&mut g, assassin, None);
    pass_pair(&mut g);
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    settle(&mut g);
    assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
    assert!(!g.object_view(victim).damaged_this_turn);
    g.restore(&saved).unwrap();
    g.begin_turn();
    assert!(!g.object_view(victim).damaged_this_turn);
    g.restore(&saved).unwrap();
    g.move_object(victim, Zone::Hand);
    g.move_object(victim, Zone::Battlefield);
    assert!(!g.object_view(victim).damaged_this_turn);
}
#[test]
fn damage_dealer_history_is_distinct_from_recipient_and_does_not_follow_a_departed_source() {
    let mut g = scenario();
    let dealer = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let recipient = named(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    let event = DamageEvent {
        source: g.damage_source(dealer),
        target: Target::Card(recipient),
        amount: 1,
        combat: false,
    };
    g.apply_damage_batch(&[
        DamageEvent { amount: 0, ..event },
        DamageEvent {
            amount: -1,
            ..event
        },
    ]);
    assert!(!g.object_view(dealer).dealt_damage_this_turn);
    assert!(!g.object_view(recipient).damaged_this_turn);
    g.apply_damage_batch(&[event]);
    assert!(g.object_view(dealer).dealt_damage_this_turn);
    assert!(!g.object_view(dealer).damaged_this_turn);
    assert!(g.object_view(recipient).damaged_this_turn);
    assert!(!g.object_view(recipient).dealt_damage_this_turn);
    let guardian = named(
        &mut g,
        Player::P0,
        "Red Guardian, Super-Soldier",
        Zone::Hand,
    );
    g.move_object(guardian, Zone::Battlefield);
    g.prepare_triggers();
    assert!(g.legal_actions().contains(&Action::ChooseTarget {
        target: Target::Card(dealer)
    }));
    assert!(!g.legal_actions().contains(&Action::ChooseTarget {
        target: Target::Card(recipient)
    }));
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(dealer),
        },
    );
    settle(&mut g);
    assert_eq!(g.object_view(dealer).zone, Zone::Graveyard);
    assert!(!g.object_view(dealer).dealt_damage_this_turn);
    g.move_object(dealer, Zone::Battlefield);
    g.apply_damage_batch(&[event]);
    assert!(!g.object_view(dealer).dealt_damage_this_turn);
    g.apply_damage_batch(&[DamageEvent {
        source: g.damage_source(dealer),
        target: Target::Player(Player::P0),
        amount: 1,
        combat: true,
    }]);
    assert!(g.object_view(dealer).dealt_damage_this_turn);
    g.begin_turn();
    assert!(!g.object_view(dealer).dealt_damage_this_turn);
    assert!(!g.object_view(recipient).damaged_this_turn);
}
