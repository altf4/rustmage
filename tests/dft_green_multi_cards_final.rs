//! Final linked-copy, exhaust, delayed-trigger and restricted-mana card outcomes.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;
fn idle(g: &Game) -> bool {
    g.state.stack.is_empty()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
        && g.state.pending == Pending::Priority
}
fn drain(g: &mut Game) {
    for _ in 0..128 {
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
                    | Action::ChooseReplacement { .. }
            )
        });
    }
    panic!("bounded final legends resolution");
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
    panic!("bounded final legends choice");
}
fn start(g: &mut Game, card: CardId, ability: u16) {
    act(
        g,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
}
fn target(g: &mut Game, card: CardId) {
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(card),
        },
    );
}
fn combat_to_end(g: &mut Game) {
    for _ in 0..128 {
        if g.state.step == Step::EndCombat && idle(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::FinishBlockers
                    | Action::AssignDamage {
                        target: Target::Player(_),
                        ..
                    }
                    | Action::PutTrigger { .. }
            )
        });
    }
    panic!("bounded combat");
}
#[test]
fn captain_howler_uses_this_discard_batch_and_delayed_draw_keeps_original_controller_after_source_leaves()
 {
    let mut g = scenario(Player::P1);
    g.state.priority = Player::P0;
    let source = put(
        &mut g,
        Player::P0,
        "Captain Howler, Sea Scourge",
        Zone::Battlefield,
    );
    let recipient = put(&mut g, Player::P1, "Riptide Gearhulk", Zone::Battlefield);
    let cards = [
        put(&mut g, Player::P0, "Forest", Zone::Hand),
        put(&mut g, Player::P0, "Mountain", Zone::Hand),
    ];
    g.discard_batch(&cards, None);
    reach(&mut g, |a| matches!(a, Action::ChooseTarget { .. }));
    target(&mut g, recipient);
    drain(&mut g);
    assert_eq!(stats(&g, recipient), (6, 5));
    assert_eq!(stats(&g, source), (5, 4));
    g.sacrifice_batch(&[(Player::P0, source)]);
    drain(&mut g);
    g.state.active = Player::P1;
    g.state.priority = Player::P1;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(&mut g, Action::Attack { card: recipient });
    act(&mut g, Action::FinishAttackers);
    combat_to_end(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 2);
    assert_eq!(g.state.players[1].cards_drawn_this_turn, 0);
    assert_eq!(g.state.players[0].life, 8);
    g.finish_cleanup();
    assert_eq!(stats(&g, recipient), (2, 5));
}
#[test]
fn captain_howler_ward_requires_two_mana_and_two_life_together_or_counters_the_spell() {
    for pay_ward in [false, true] {
        let mut g = scenario(Player::P1);
        let source = put(
            &mut g,
            Player::P0,
            "Captain Howler, Sea Scourge",
            Zone::Battlefield,
        );
        let spell = put(&mut g, Player::P1, "Lightning Strike", Zone::Hand);
        g.state.players[1].mana = [0, 0, 0, 1, 0, 3];
        cast(&mut g, spell, Some(Target::Card(source)));
        reach(&mut g, |a| matches!(a, Action::DeclineResolutionMana));
        if pay_ward {
            choose(&mut g, |a| matches!(a, Action::PayResolutionMana { .. }));
        } else {
            act(&mut g, Action::DeclineResolutionMana);
        }
        drain(&mut g);
        assert_eq!(g.state.players[1].life, if pay_ward { 18 } else { 20 });
        assert_eq!(g.object_view(source).damage, if pay_ward { 3 } else { 0 });
        assert_eq!(
            g.state.players[1].mana.iter().sum::<u32>(),
            if pay_ward { 0 } else { 2 }
        );
    }
}
#[test]
fn ketramose_attack_and_block_require_seven_physical_cards_in_all_players_exile() {
    for count in [6, 7] {
        let mut g = scenario(Player::P0);
        let source = put(
            &mut g,
            Player::P0,
            "Ketramose, the New Dawn",
            Zone::Battlefield,
        );
        for i in 0..count {
            put(
                &mut g,
                if i % 2 == 0 { Player::P0 } else { Player::P1 },
                "Forest",
                Zone::Exile,
            );
        }
        assert!(
            g.keywords(source).contains(
                Keywords::MENACE
                    .union(Keywords::LIFELINK)
                    .union(Keywords::INDESTRUCTIBLE)
            )
        );
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        assert_eq!(
            g.legal_actions().contains(&Action::Attack { card: source }),
            count == 7
        );
        g.state.active = Player::P1;
        g.state.priority = Player::P1;
        let attacker = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        drain(&mut g);
        assert_eq!(g.can_block_pair(source, attacker), count == 7);
    }
}
#[test]
fn ketramose_observes_one_global_graveyard_exile_batch_only_on_its_controllers_turn() {
    for active in [Player::P0, Player::P1] {
        let mut g = scenario(active);
        put(
            &mut g,
            Player::P0,
            "Ketramose, the New Dawn",
            Zone::Battlefield,
        );
        for p in [Player::P0, Player::P1] {
            put(&mut g, p, "Forest", Zone::Graveyard);
            put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
        }
        let spell = put(&mut g, active, "Rest in Peace", Zone::Hand);
        cast(&mut g, spell, None);
        drain(&mut g);
        assert_eq!(
            g.state.players[0].cards_drawn_this_turn,
            u64::from(active == Player::P0)
        );
        assert_eq!(
            g.state.players[0].life,
            if active == Player::P0 { 19 } else { 20 }
        );
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Exile)
                .count(),
            4
        );
    }
}
#[test]
fn loot_has_three_independent_exhaust_instances_with_tap_and_the_printed_outputs() {
    let mut g = scenario(Player::P0);
    let source = put(
        &mut g,
        Player::P0,
        "Loot, the Pathfinder",
        Zone::Battlefield,
    );
    assert_eq!(stats(&g, source), (2, 4));
    assert!(
        g.keywords(source).contains(
            Keywords::DOUBLE_STRIKE
                .union(Keywords::VIGILANCE)
                .union(Keywords::HASTE)
        )
    );
    g.state.players[0].mana = [0, 0, 0, 0, 1, 0];
    let production = [0, 3, 0, 0, 0, 0];
    act(
        &mut g,
        Action::AnnounceMana {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
            production,
        },
    );
    pay(&mut g);
    assert_eq!(g.state.players[0].mana, production);
    assert!(g.object_view(source).tapped);
    g.state.objects[source as usize].tapped = false;
    g.state.players[0].mana[4] = 1;
    reject(
        &mut g,
        Action::AnnounceMana {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
            production,
        },
    );
    start(&mut g, source, 1);
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 3);
    g.state.objects[source as usize].tapped = false;
    g.state.players[0].mana[3] = 1;
    start(&mut g, source, 2);
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.state.players[1].life, 17);
    g.state.objects[source as usize].tapped = false;
    g.state.players[0].mana[3] = 1;
    for ability in [1, 2] {
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability,
                counter_payment: Counters::default(),
            },
        );
    }
}
#[test]
fn mendicant_power_counts_itself_and_artifacts_and_max_speed_copy_is_optional_and_becomes_a_token()
{
    for (speed, accept, expected) in [(3, false, 1), (4, false, 1), (4, true, 2)] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Mendicant Core, Guidelight", Zone::Hand);
        cast(&mut g, source, None);
        drain(&mut g);
        assert_eq!(stats(&g, source), (1, 3));
        assert_eq!(g.state.players[0].speed, 1);
        g.state.players[0].speed = speed;
        let spell = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Hand);
        cast(&mut g, spell, None);
        if speed == 4 {
            reach(&mut g, |a| matches!(a, Action::DeclineResolutionMana));
            if accept {
                choose(&mut g, |a| matches!(a, Action::PayResolutionMana { .. }));
            } else {
                act(&mut g, Action::DeclineResolutionMana);
            }
        }
        drain(&mut g);
        let boots: Vec<_> = g
            .state
            .objects
            .iter()
            .filter(|o| {
                o.zone == Zone::Battlefield && o.card == Card::from_name("Swiftfoot Boots").unwrap()
            })
            .collect();
        assert_eq!(boots.len(), expected);
        assert_eq!(
            boots.iter().filter(|o| o.is_token()).count(),
            usize::from(accept)
        );
        assert_eq!(stats(&g, source), (1 + expected as i128, 3));
    }
}
#[test]
fn mimeoplasm_actual_cast_exiles_only_own_creatures_and_retains_linked_copy_activation_across_copies()
 {
    let mut g = scenario(Player::P0);
    let first = put(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let second = put(&mut g, Player::P0, "Llanowar Elves", Zone::Graveyard);
    let bad = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Graveyard);
    let foreign = put(&mut g, Player::P1, "Hill Giant", Zone::Graveyard);
    let unrelated = put(&mut g, Player::P0, "Serra Angel", Zone::Exile);
    let source = put(&mut g, Player::P0, "Mimeoplasm, Revered One", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: source });
    act(&mut g, Action::ChooseX { value: 2 });
    pay(&mut g);
    reach(&mut g, |a| matches!(a, Action::ChooseEntryExile { .. }));
    for id in [bad, foreign] {
        let object = g.object_ref(id);
        reject(&mut g, Action::ChooseEntryExile { object });
    }
    for id in [first, second] {
        let object = g.object_ref(id);
        act(&mut g, Action::ChooseEntryExile { object });
    }
    drain(&mut g);
    assert_eq!(stats(&g, source), (6, 6));
    assert_eq!(g.counter_count(source, CounterKind::PlusOne), 6);
    start(&mut g, source, 0);
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(unrelated),
        },
    );
    target(&mut g, second);
    pay(&mut g);
    drain(&mut g);
    assert_eq!(stats(&g, source), (6, 6));
    assert!(g.has_subtype(source, "Elf"));
    let object = g.object_ref(source);
    act(
        &mut g,
        Action::AnnounceRetainedCopyAbility { source: object },
    );
    target(&mut g, first);
    pay(&mut g);
    drain(&mut g);
    assert_eq!(stats(&g, source), (6, 6));
    assert!(g.has_subtype(source, "Bear"));
    assert_eq!(g.counter_count(source, CounterKind::PlusOne), 6);
}
#[test]
fn mimeoplasm_cast_x_zero_exiles_nothing_and_dies_as_zero_zero() {
    let mut g = scenario(Player::P0);
    let grave = put(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let source = put(&mut g, Player::P0, "Mimeoplasm, Revered One", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: source });
    act(&mut g, Action::ChooseX { value: 0 });
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    assert_eq!(g.object_view(grave).zone, Zone::Graveyard);
}
#[test]
fn redshift_power_mana_is_one_color_and_pays_activations_but_cannot_cast_a_spell() {
    let mut g = scenario(Player::P0);
    let source = put(
        &mut g,
        Player::P0,
        "Redshift, Rocketeer Chief",
        Zone::Battlefield,
    );
    g.add_counter(source, CounterKind::PlusOne, 1);
    let mascot = put(&mut g, Player::P0, "District Mascot", Zone::Battlefield);
    g.add_counter(mascot, CounterKind::PlusOne, 2);
    let enemy = put(&mut g, Player::P1, "Swiftfoot Boots", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Llanowar Elves", Zone::Hand);
    g.state.players[0].mana = [0; 6];
    let production = [0, 0, 0, 0, 3, 0];
    act(
        &mut g,
        Action::ActivateMana {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
            payment: [0; 6],
            production,
        },
    );
    assert_eq!(g.state.players[0].mana, production);
    assert!(g.object_view(source).tapped);
    assert!(g.keywords(source).contains(Keywords::VIGILANCE));
    reject(&mut g, Action::AnnounceCast { card: spell });
    act(
        &mut g,
        Action::AnnounceActivate {
            card: mascot,
            ability: 0,
            counter_payment: Counters::of(CounterKind::PlusOne, 2),
        },
    );
    target(&mut g, enemy);
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].mana[4], 1);
}
#[test]
fn redshift_exhaust_private_selection_commits_one_simultaneous_permanent_batch_and_cannot_repeat() {
    let mut g = scenario(Player::P0);
    let source = put(
        &mut g,
        Player::P0,
        "Redshift, Rocketeer Chief",
        Zone::Battlefield,
    );
    let cards = [
        "Thunderous Velocipede",
        "Grizzly Bears",
        "Forest",
        "Swiftfoot Boots",
    ]
    .map(|n| put(&mut g, Player::P0, n, Zone::Hand));
    let bad = put(&mut g, Player::P0, "Lightning Strike", Zone::Hand);
    start(&mut g, source, 1);
    pay(&mut g);
    reach(&mut g, |a| matches!(a, Action::ChooseHandCard { .. }));
    assert!(g.observe(Player::P1).hand_choice.is_none());
    let object = Some(g.object_ref(bad));
    reject(&mut g, Action::ChooseHandCard { object });
    for id in cards {
        let object = Some(g.object_ref(id));
        act(&mut g, Action::ChooseHandCard { object });
        assert_eq!(g.object_view(id).zone, Zone::Hand);
    }
    act(&mut g, Action::ChooseHandCard { object: None });
    drain(&mut g);
    for id in cards {
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
    }
    assert_eq!(
        g.counter_count(cards[1], CounterKind::PlusOne),
        0,
        "coentering provider cannot add to another entrant"
    );
    assert_eq!(g.state.players[0].spells_cast_this_turn, 0);
    assert_eq!(g.object_view(bad).zone, Zone::Hand);
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: source,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
}
#[test]
fn sita_varma_adds_x_counters_then_optionally_samples_its_power_for_other_creatures_base_stats() {
    for accept in [false, true] {
        let mut g = scenario(Player::P0);
        let source = put(
            &mut g,
            Player::P0,
            "Sita Varma, Masked Racer",
            Zone::Battlefield,
        );
        let own = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        g.add_counter(own, CounterKind::PlusOne, 1);
        let enemy = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
        start(&mut g, source, 0);
        act(&mut g, Action::ChooseX { value: 2 });
        pay(&mut g);
        reach(&mut g, |a| matches!(a, Action::ChooseOptional { .. }));
        assert_eq!(stats(&g, source), (4, 5));
        act(&mut g, Action::ChooseOptional { accept });
        drain(&mut g);
        assert_eq!(stats(&g, own), if accept { (5, 5) } else { (3, 3) });
        assert_eq!(stats(&g, enemy), (2, 2));
        g.state.objects[source as usize].growth.power = 7;
        assert_eq!(stats(&g, own), if accept { (5, 5) } else { (3, 3) });
        g.finish_cleanup();
        assert_eq!(stats(&g, own), (3, 3));
        g.state.priority = Player::P0;
        g.state.players[0].mana = [8; 6];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
    }
}
#[test]
fn winter_declared_artifact_exile_count_weakens_other_nonartifact_creatures_of_both_controllers() {
    for x in [0, 2] {
        let mut g = scenario(Player::P0);
        let source = put(
            &mut g,
            Player::P0,
            "Winter, Cursed Rider",
            Zone::Battlefield,
        );
        let own = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
        let enemy = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
        let artifact = put(&mut g, Player::P0, "Gleaming Barrier", Zone::Battlefield);
        let materials = [
            put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Graveyard),
            put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Graveyard),
        ];
        let bad = put(&mut g, Player::P0, "Forest", Zone::Graveyard);
        start(&mut g, source, 0);
        let object = g.object_ref(bad);
        reject(&mut g, Action::ChooseAdditionalCost { object });
        for &id in materials.iter().take(x) {
            let object = g.object_ref(id);
            act(&mut g, Action::ChooseAdditionalCost { object });
        }
        act(&mut g, Action::FinishAdditionalCostGroup);
        pay(&mut g);
        drain(&mut g);
        assert_eq!(stats(&g, source), (3, 2));
        assert!(g.object_view(source).tapped);
        assert_eq!(stats(&g, own), (3 - x as i128, 3 - x as i128));
        assert_eq!(stats(&g, artifact), (0, 4));
        assert_eq!(
            g.object_view(enemy).zone,
            if x == 2 {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        for id in materials {
            assert_eq!(
                g.object_view(id).zone,
                if x == 2 { Zone::Exile } else { Zone::Graveyard }
            );
        }
        g.state.objects[source as usize].tapped = false;
        g.state.players[0].mana = [8; 6];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
    }
}
#[test]
fn winter_has_one_native_life_ward_and_grants_one_independent_life_ward_to_each_controlled_artifact()
 {
    for artifact in [false, true] {
        for pay_ward in [false, true] {
            let mut g = scenario(Player::P1);
            let source = put(
                &mut g,
                Player::P0,
                "Winter, Cursed Rider",
                Zone::Battlefield,
            );
            let target_id = if artifact {
                put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield)
            } else {
                source
            };
            let spell = put(
                &mut g,
                Player::P1,
                if artifact { "Broken Wings" } else { "Murder" },
                Zone::Hand,
            );
            cast(&mut g, spell, Some(Target::Card(target_id)));
            reach(&mut g, |a| matches!(a, Action::DeclineResolutionLife));
            act(
                &mut g,
                if pay_ward {
                    Action::PayResolutionLife
                } else {
                    Action::DeclineResolutionLife
                },
            );
            drain(&mut g);
            assert_eq!(g.state.players[1].life, if pay_ward { 18 } else { 20 });
            assert_eq!(
                g.object_view(target_id).zone,
                if pay_ward {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}

#[test]
fn ketramose_also_observes_a_physical_battlefield_artifact_exiled_on_its_controllers_turn() {
    let mut g = scenario(Player::P0);
    let source = put(
        &mut g,
        Player::P0,
        "Ketramose, the New Dawn",
        Zone::Battlefield,
    );
    let artifact = put(&mut g, Player::P1, "Swiftfoot Boots", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Explosive Getaway", Zone::Hand);
    announce(&mut g, spell, &[Target::Card(artifact)]);
    drain(&mut g);
    assert_eq!(g.object_view(artifact).zone, Zone::Exile);
    assert_eq!(g.object_view(source).zone, Zone::Battlefield);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    assert_eq!(g.state.players[0].life, 19);
}
