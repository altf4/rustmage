//! Live layers, entry/attack history and speed-dependent legendary creatures.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;
fn drain(g: &mut Game) {
    for _ in 0..96 {
        if g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.pending == Pending::Priority
        {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::FinishTargets
                    | Action::FinishStackTargets
                    | Action::Discard { .. }
                    | Action::ChooseReplacement { .. }
            )
        });
    }
    panic!("bounded legends resolution");
}
fn reach(g: &mut Game, wanted: impl Fn(&Action) -> bool) {
    for _ in 0..96 {
        if g.legal_actions().iter().any(&wanted) {
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
    panic!("bounded legends choice");
}
fn attack(g: &mut Game, cards: &[CardId]) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    for &card in cards {
        act(g, Action::Attack { card });
    }
    act(g, Action::FinishAttackers);
    drain(g);
}
#[test]
fn aatchik_deduplicates_artifact_creatures_then_observes_each_other_controlled_insect_death() {
    let mut g = scenario(Player::P0);
    for name in [
        "Gleaming Barrier",
        "Grizzly Bears",
        "Swiftfoot Boots",
        "Forest",
    ] {
        put(&mut g, Player::P0, name, Zone::Graveyard);
    }
    put(&mut g, Player::P1, "Grizzly Bears", Zone::Graveyard);
    let source = put(&mut g, Player::P0, "Aatchik, Emerald Radian", Zone::Hand);
    cast(&mut g, source, None);
    drain(&mut g);
    let insects: Vec<_> = g
        .state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield && o.is_token() && g.has_subtype(id as CardId, "Insect"))
                .then_some(id as CardId)
        })
        .collect();
    assert_eq!(insects.len(), 3);
    for &id in &insects {
        assert_eq!(stats(&g, id), (1, 1));
        assert_eq!(g.object_view(id).colors, 16);
    }
    g.sacrifice_batch(&[(Player::P0, insects[0]), (Player::P0, insects[1])]);
    drain(&mut g);
    assert_eq!(g.counter_count(source, CounterKind::PlusOne), 2);
    assert_eq!(g.state.players[1].life, 18);
    g.sacrifice_batch(&[(Player::P0, source)]);
    drain(&mut g);
    assert_eq!(g.state.players[1].life, 18);
}
#[test]
fn far_fortune_starts_speed_and_deals_attack_trigger_damage_even_when_another_creature_attacks() {
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Far Fortune, End Boss", Zone::Hand);
    cast(&mut g, source, None);
    drain(&mut g);
    assert_eq!(g.state.players[0].speed, 1);
    assert_eq!(stats(&g, source), (4, 5));
    let bear = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    attack(&mut g, &[bear]);
    assert_eq!(g.state.players[1].life, 19);
    assert_eq!(g.state.players[0].speed, 2);
}
#[test]
fn far_fortune_max_speed_adds_one_only_to_controlled_damage_to_opposing_players_or_permanents() {
    for (speed, caster, recipient, expected) in [
        (3, Player::P0, Player::P1, 3),
        (4, Player::P0, Player::P1, 4),
        (4, Player::P0, Player::P0, 3),
        (4, Player::P1, Player::P0, 3),
    ] {
        let mut g = scenario(caster);
        put(
            &mut g,
            Player::P0,
            "Far Fortune, End Boss",
            Zone::Battlefield,
        );
        g.state.players[0].speed = speed;
        let creature = put(&mut g, recipient, "Grizzly Bears", Zone::Battlefield);
        g.add_counter(creature, CounterKind::PlusOne, 4);
        let spell = put(&mut g, caster, "Lightning Strike", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(creature)));
        drain(&mut g);
        assert_eq!(g.object_view(creature).damage, expected);
    }
}
#[test]
fn gastal_entry_damage_and_life_start_speed_before_live_max_speed_keywords() {
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Gastal Thrillseeker", Zone::Hand);
    cast(&mut g, source, None);
    reach(&mut g, |a| matches!(a, Action::ChooseTarget { .. }));
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(Player::P0),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    drain(&mut g);
    assert_eq!(g.state.players[0].life, 21);
    assert_eq!(g.state.players[1].life, 19);
    assert_eq!(g.state.players[0].speed, 2);
    assert_eq!(stats(&g, source), (2, 3));
    assert!(
        !g.keywords(source)
            .intersects(Keywords::DEATHTOUCH.union(Keywords::HASTE))
    );
    g.state.players[0].speed = 4;
    assert!(
        g.keywords(source)
            .contains(Keywords::DEATHTOUCH.union(Keywords::HASTE))
    );
    g.state.objects[source as usize].controller = Player::P1;
    assert!(
        !g.keywords(source)
            .intersects(Keywords::DEATHTOUCH.union(Keywords::HASTE))
    );
}
#[test]
fn fearless_grants_vehicle_haste_and_draws_three_discards_two_only_when_pirate_and_vehicle_attacked()
 {
    for mode in 0..3 {
        let mut g = scenario(Player::P0);
        let source = put(
            &mut g,
            Player::P0,
            "Fearless Swashbuckler",
            Zone::Battlefield,
        );
        let vehicle = put(&mut g, Player::P0, "Cultivator's Caravan", Zone::Hand);
        cast(&mut g, vehicle, None);
        drain(&mut g);
        let rider = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: vehicle,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        if matches!(g.state.pending, Pending::PayCosts { .. }) {
            pay(&mut g);
        }
        let object = g.object_ref(rider);
        act(&mut g, Action::ChooseCrewer { object });
        act(&mut g, Action::FinishCrew);
        drain(&mut g);
        assert!(g.keywords(source).contains(Keywords::HASTE));
        assert!(g.keywords(vehicle).contains(Keywords::HASTE));
        let enemy = put(
            &mut g,
            Player::P1,
            "Cultivator's Caravan",
            Zone::Battlefield,
        );
        assert!(!g.keywords(enemy).contains(Keywords::HASTE));
        put(&mut g, Player::P0, "Forest", Zone::Hand);
        put(&mut g, Player::P0, "Forest", Zone::Hand);
        let attackers = if mode == 0 {
            vec![source]
        } else if mode == 1 {
            vec![vehicle]
        } else {
            vec![source, vehicle]
        };
        attack(&mut g, &attackers);
        assert_eq!(
            g.state.players[0].cards_drawn_this_turn,
            if mode == 2 { 3 } else { 0 }
        );
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.owner == Player::P0 && o.zone == Zone::Graveyard)
                .count(),
            if mode == 2 { 2 } else { 0 }
        );
    }
}
#[test]
fn kolodin_filters_its_own_entry_and_directly_saddles_mounts_animates_vehicles_with_haste_until_cleanup()
 {
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Kolodin, Triumph Caster", Zone::Hand);
    cast(&mut g, source, None);
    drain(&mut g);
    assert!(!g.object_view(source).saddled);
    assert_eq!(g.object_view(source).kinds & 16, 0);
    let mount = put(&mut g, Player::P0, "Alacrian Jaguar", Zone::Hand);
    cast(&mut g, mount, None);
    drain(&mut g);
    let vehicle = put(&mut g, Player::P0, "Veloheart Bike", Zone::Hand);
    cast(&mut g, vehicle, None);
    drain(&mut g);
    assert!(g.object_view(mount).saddled);
    assert!(g.keywords(mount).contains(Keywords::HASTE));
    assert!(g.is_creature(vehicle));
    assert_eq!(stats(&g, vehicle), (4, 2));
    assert!(g.keywords(vehicle).contains(Keywords::HASTE));
    g.finish_cleanup();
    assert!(!g.object_view(mount).saddled);
    assert!(!g.is_creature(vehicle));
    assert!(g.keywords(vehicle).contains(Keywords::HASTE));
}
#[test]
fn march_adds_ooze_without_removing_types_sets_live_six_six_and_triggers_only_on_opponents_off_turn_casts()
 {
    let mut g = scenario(Player::P0);
    let own = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    g.add_counter(own, CounterKind::PlusOne, 1);
    let enemy = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let source = put(&mut g, Player::P0, "March of the World Ooze", Zone::Hand);
    cast(&mut g, source, None);
    drain(&mut g);
    assert_eq!(stats(&g, own), (7, 7));
    assert!(g.has_subtype(own, "Bear") && g.has_subtype(own, "Ooze"));
    assert_eq!(stats(&g, enemy), (2, 2));
    assert!(!g.has_subtype(enemy, "Ooze"));
    for off_turn in [true, false] {
        g.state.active = if off_turn { Player::P0 } else { Player::P1 };
        g.state.priority = Player::P1;
        let spell = put(&mut g, Player::P1, "Lightning Strike", Zone::Hand);
        cast(&mut g, spell, Some(Target::Player(Player::P0)));
        drain(&mut g);
        let elephants: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, o)| {
                (o.zone == Zone::Battlefield
                    && o.is_token()
                    && g.has_subtype(id as CardId, "Elephant"))
                .then_some(id as CardId)
            })
            .collect();
        assert_eq!(elephants.len(), 1);
        assert_eq!(stats(&g, elephants[0]), (6, 6));
        assert!(g.has_subtype(elephants[0], "Ooze"));
    }
}
#[test]
fn sab_sunen_counts_all_counter_kinds_for_attack_and_block_including_zero_even() {
    for counters in 0..=2 {
        let mut g = scenario(Player::P0);
        let source = put(
            &mut g,
            Player::P0,
            "Sab-Sunen, Luxa Embodied",
            Zone::Battlefield,
        );
        if counters > 0 {
            g.add_counter(source, CounterKind::Charge, counters);
        }
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        assert_eq!(
            g.legal_actions().contains(&Action::Attack { card: source }),
            counters % 2 == 0
        );
        assert!(
            g.keywords(source).contains(
                Keywords::REACH
                    .union(Keywords::TRAMPLE)
                    .union(Keywords::INDESTRUCTIBLE)
            )
        );
        g.state.active = Player::P1;
        g.state.priority = Player::P1;
        g.state.pending = Pending::Attackers;
        let enemy = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
        act(&mut g, Action::Attack { card: enemy });
        act(&mut g, Action::FinishAttackers);
        drain(&mut g);
        assert_eq!(g.can_block_pair(source, enemy), counters % 2 == 0);
    }
}
#[test]
fn sab_sunen_first_main_adds_one_then_draws_only_if_post_placement_total_is_odd() {
    for charge in [0, 1] {
        let mut g = scenario(Player::P0);
        let source = put(
            &mut g,
            Player::P0,
            "Sab-Sunen, Luxa Embodied",
            Zone::Battlefield,
        );
        if charge > 0 {
            g.add_counter(source, CounterKind::Charge, charge);
        }
        g.begin_step(Step::Main1);
        g.prepare_triggers();
        drain(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
        assert_eq!(
            g.state.players[0].cards_drawn_this_turn,
            if charge == 0 { 2 } else { 0 }
        );
        assert_eq!(stats(&g, source), (7, 7));
    }
}
#[test]
fn samut_live_speed_buffs_only_other_controlled_creatures_and_discounts_only_noncreature_generic_cost()
 {
    for speed in [0, 1, 3, 4] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Samut, the Driving Force", Zone::Hand);
        cast(&mut g, source, None);
        drain(&mut g);
        assert_eq!(g.state.players[0].speed, 1);
        g.state.players[0].speed = speed;
        let own = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let enemy = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
        assert_eq!(stats(&g, source), (4, 5));
        assert!(
            g.keywords(source).contains(
                Keywords::FIRST_STRIKE
                    .union(Keywords::VIGILANCE)
                    .union(Keywords::HASTE)
            )
        );
        assert_eq!(stats(&g, own), (2 + i128::from(speed), 2));
        assert_eq!(stats(&g, enemy), (2, 2));
        let spell = put(&mut g, Player::P0, "Divination", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        assert_eq!(
            g.mana_payment_domain().unwrap().generic,
            2u32.saturating_sub(u32::from(speed))
        );
        pay(&mut g);
        drain(&mut g);
        let creature = put(&mut g, Player::P0, "Hill Giant", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: creature });
        assert_eq!(g.mana_payment_domain().unwrap().generic, 3);
        pay(&mut g);
        drain(&mut g);
    }
}
