//! Printed storied outcomes plus one representative owned designation root.
//! Rules: https://magic.wizards.com/en/news/feature/the-hobbit-release-notes
//! Storied precedes SBAs, counts a union of distinct permanents, and lasts forever.
use super::*;

fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let card = put(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = [12; 6];
    cast(g, card, None);
    settle(g);
    card
}
fn enduring(g: &Game, p: Player) -> bool {
    let owner = g.state.players[p.index()].enduring_story;
    for viewer in [Player::P0, Player::P1] {
        assert_eq!(g.observe(viewer).players[p.index()].enduring_story, owner);
        assert_eq!(
            serde_json::to_value(g.observe(viewer)).unwrap()["players"][p.index()]["enduring_story"],
            owner
        );
    }
    owner
}
fn activate(g: &mut Game, card: CardId, target: Option<Target>) {
    act(
        g,
        Action::AnnounceActivate {
            card,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    if let Some(target) = target {
        act(g, Action::ChooseTarget { target });
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}

#[test]
fn hob_storied_paid_consumers_threshold_bonuses_and_persistent_public_designation() {
    for p in [Player::P0, Player::P1] {
        for (name, before, after, keyword) in [
            ("Ori, Keeper of Songs", (3, 3), (4, 3), Keywords::VIGILANCE),
            ("Óin the Brave", (1, 3), (2, 3), Keywords::HASTE),
            ("Bombur, Gentle Dreamer", (5, 3), (5, 3), Keywords::NONE),
        ] {
            let mut g = scenario(p);
            let source = paid(&mut g, p, name);
            assert_eq!(stats(&g, source), before, "{name}");
            assert!(!enduring(&g, p));
            if keyword != Keywords::NONE {
                assert!(!g.object_view(source).keywords.contains(keyword));
            }
            let first = paid(&mut g, p, "Goldvein Pick");
            assert!(!enduring(&g, p));
            let second = paid(&mut g, p, "Lotus Ring");
            assert!(enduring(&g, p), "{name}");
            assert!(!enduring(&g, p.other()));
            assert_eq!(stats(&g, source), after, "{name}");
            assert!(g.object_view(source).keywords.contains(keyword));
            g.move_objects_raw(&[first, second], Zone::Exile);
            assert!(enduring(&g, p));
            assert_eq!(stats(&g, source), after);
            g.record_control(g.object_ref(source), p.other(), None);
            assert_eq!(stats(&g, source), before, "bonus uses current controller");
            if keyword != Keywords::NONE {
                assert!(!g.object_view(source).keywords.contains(keyword));
            }
            g.move_object(source, Zone::Exile);
            g.state.active = p;
            g.begin_turn();
            assert!(
                enduring(&g, p),
                "designation survives source and turn changes"
            );
        }
    }
}

#[test]
fn hob_storied_distinct_mixed_union_saga_tokens_and_source_requirement() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        // Three artifacts without storied grant nothing, nor is a past threshold saved.
        let artifacts: Vec<_> = (0..3)
            .map(|_| put(&mut g, p, "Goldvein Pick", Zone::Battlefield))
            .collect();
        g.refresh_controllers();
        assert!(!enduring(&g, p));
        g.move_objects_raw(&artifacts, Zone::Exile);
        let source = paid(&mut g, p, "Ori, Keeper of Songs");
        assert!(!enduring(&g, p));
        let multiple_qualities = put(&mut g, p, "Mjölnir, Hammer of Thor", Zone::Battlefield);
        g.refresh_controllers();
        assert!(!enduring(&g, p), "one legendary artifact counts only once");
        put(&mut g, p.other(), "Armor Wars", Zone::Battlefield);
        g.refresh_controllers();
        assert!(!enduring(&g, p), "opponent's Saga does not count");
        let saga = put(&mut g, p, "Armor Wars", Zone::Hand);
        g.move_object(saga, Zone::Battlefield);
        assert!(
            enduring(&g, p),
            "legendary + legendary artifact + Saga are three"
        );
        g.move_objects_raw(&[source, multiple_qualities, saga], Zone::Exile);
        assert!(enduring(&g, p));

        let mut g = scenario(p);
        let source = paid(&mut g, p, "Ori, Keeper of Songs");
        g.create_tokens(p, Card::MshDoombotToken, 2, false);
        assert!(enduring(&g, p), "artifact tokens count");
        assert_eq!(stats(&g, source), (4, 3));
    }
}

#[test]
fn hob_storied_suppressed_source_cannot_award_and_returning_ability_can() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = paid(&mut g, p, "Ori, Keeper of Songs");
        let aura = put(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, aura, Some(Target::Card(source)));
        settle(&mut g);
        let first = paid(&mut g, p, "Goldvein Pick");
        paid(&mut g, p, "Lotus Ring");
        assert!(!enduring(&g, p));
        let disenchant = put(&mut g, p, "Disenchant", Zone::Hand);
        cast(&mut g, disenchant, Some(Target::Card(aura)));
        settle(&mut g);
        assert!(enduring(&g, p));
        assert_eq!(stats(&g, source), (4, 3));
        let second_aura = put(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, second_aura, Some(Target::Card(source)));
        settle(&mut g);
        assert!(
            enduring(&g, p),
            "designation is not an ability on its source"
        );
        assert_eq!(
            stats(&g, source),
            (1, 1),
            "source's printed bonus is suppressed"
        );
        assert!(!g.object_view(source).keywords.contains(Keywords::VIGILANCE));
        g.move_object(first, Zone::Exile);
        assert!(enduring(&g, p));
    }
}

#[test]
fn hob_storied_acquires_before_legend_choice_and_owned_branches_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let original = paid(&mut g, p, "Ori, Keeper of Songs");
        paid(&mut g, p, "Goldvein Pick");
        let duplicate = put(&mut g, p, "Ori, Keeper of Songs", Zone::Hand);
        cast(&mut g, duplicate, None);
        for _ in 0..12 {
            if matches!(g.state.pending, Pending::Legend { .. }) {
                break;
            }
            act(&mut g, Action::Pass);
        }
        assert!(matches!(g.state.pending, Pending::Legend { player } if player == p));
        assert!(enduring(&g, p), "grant precedes the legend rule");
        super::contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let mut child = g.clone();
        let original_ref = child.object_ref(original);
        act(
            &mut child,
            Action::KeepLegend {
                object: original_ref,
            },
        );
        settle(&mut child);
        assert_eq!(child.object_view(duplicate).zone, Zone::Graveyard);
        assert!(enduring(&child, p));
        child.restore(&saved).unwrap();
        let duplicate_ref = child.object_ref(duplicate);
        act(
            &mut child,
            Action::KeepLegend {
                object: duplicate_ref,
            },
        );
        settle(&mut child);
        assert_eq!(child.object_view(original).zone, Zone::Graveyard);
        assert!(enduring(&child, p));
    }
}

#[test]
fn hob_storied_oin_pays_discard_mana_and_tap_with_live_haste_legality() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = paid(&mut g, p, "Óin the Brave");
        let secret = put(&mut g, p, "Shock", Zone::Hand);
        let announce = Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject(&mut g, announce.clone());
        paid(&mut g, p, "Goldvein Pick");
        paid(&mut g, p, "Lotus Ring");
        assert!(g.object_view(source).summoning_sick);
        assert!(g.object_view(source).keywords.contains(Keywords::HASTE));
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, announce.clone());
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        act(&mut g, announce);
        let enemy = put(&mut g, p.other(), "Forest", Zone::Hand);
        let enemy_ref = g.object_ref(enemy);
        reject(&mut g, Action::ChooseAdditionalCost { object: enemy_ref });
        assert!(g.observe(p.other()).objects.iter().all(|o| o.id != secret));
        super::contracts::branch_contract(g.clone());
        let secret_ref = g.object_ref(secret);
        act(&mut g, Action::ChooseAdditionalCost { object: secret_ref });
        pay(&mut g);
        assert_eq!(
            g.object_view(secret).zone,
            Zone::Graveyard,
            "discard is a cost"
        );
        assert!(g.object_view(source).tapped);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(g.hand_count(p), 1);
    }
}

#[test]
fn hob_storied_bombur_conditional_untap_stun_and_effect_untap() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bombur = paid(&mut g, p, "Bombur, Gentle Dreamer");
        g.state.objects[bombur as usize].tapped = true;
        g.add_counter(bombur, CounterKind::Stun, 1);
        g.untap_active_player();
        assert!(g.object_view(bombur).tapped);
        assert_eq!(
            g.counter_count(bombur, CounterKind::Stun),
            1,
            "prohibition precedes stun"
        );
        g.untap_objects(&[bombur], untap_rules::UntapCause::Effect);
        assert_eq!(g.counter_count(bombur, CounterKind::Stun), 0);
        g.untap_objects(&[bombur], untap_rules::UntapCause::Effect);
        assert!(
            !g.object_view(bombur).tapped,
            "printed restriction allows effect untapping"
        );
        paid(&mut g, p, "Goldvein Pick");
        paid(&mut g, p, "Lotus Ring");
        g.state.objects[bombur as usize].tapped = true;
        g.untap_active_player();
        assert!(
            !g.object_view(bombur).tapped,
            "enduring story permits normal untapping"
        );
    }
}

#[test]
fn hob_storied_real_mana_sacrifice_refreshes_ability_return_but_planning_is_pure() {
    for p in [Player::P0, Player::P1] {
        for extra in [false, true] {
            let mut g = scenario(p);
            let source = paid(&mut g, p, "Ori, Keeper of Songs");
            let wasp = put(&mut g, p, "The Wondrous Wasp", Zone::Hand);
            cast(&mut g, wasp, None);
            let target = Target::Card(source);
            for _ in 0..24 {
                if g.legal_actions().contains(&Action::ChooseTarget { target }) {
                    act(&mut g, Action::ChooseTarget { target });
                    if g.legal_actions().contains(&Action::FinishTargets) {
                        act(&mut g, Action::FinishTargets);
                    }
                    break;
                }
                assert_eq!(g.state.pending, Pending::Priority);
                act(&mut g, Action::Pass);
            }
            settle(&mut g);
            assert!(!g.printed_abilities_available(source));
            let ring = paid(&mut g, p, "Lotus Ring");
            activate(&mut g, ring, Some(Target::Card(wasp)));
            settle(&mut g);
            if extra {
                let treasure = Card::from_name("@token/b60f6cd3b44b22bd289fe4e2").unwrap();
                g.create_tokens(p, treasure, 1, false);
            }
            assert!(!enduring(&g, p), "suppressed storied never awards");
            g.state.objects[wasp as usize].summoning_sick = false;
            g.state.players[p.index()].mana = [0; 6];
            let saved = g.snapshot();
            assert!(g.can_fund(FundingGoal {
                actor: p,
                required: [0, 0, 0, 3, 0, 0],
                generic: 0,
                reserve: None,
                source: None
            }));
            assert_eq!(
                g.snapshot(),
                saved,
                "funding speculation does not grant designation"
            );
            let action = Action::TapGrantedMana {
                card: wasp,
                grantor: g.object_ref(ring),
                production: [0, 0, 0, 3, 0, 0],
            };
            let revision = g.revision();
            let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
            let mut raw = g.clone();
            let mut indexed = g.clone();
            drop(g);
            act(&mut raw, action.clone());
            indexed.apply_index(p, revision, index).unwrap();
            assert_eq!(raw.snapshot(), indexed.snapshot());
            assert_eq!(enduring(&raw, p), extra);
            assert_eq!(raw.object_view(wasp).zone, Zone::Graveyard);
            assert!(raw.printed_abilities_available(source));
            indexed.restore(&saved).unwrap();
            assert!(!enduring(&indexed, p));
            act(&mut indexed, action);
            assert_eq!(indexed.snapshot(), raw.snapshot());
        }
    }
}

#[test]
fn hob_storied_transient_threshold_inside_one_typed_resolution() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = paid(&mut g, p, "Óin the Brave");
    paid(&mut g, p, "Goldvein Pick");
    assert!(!enduring(&g, p));
    // A bounded typed core fixture, independent of any card's printed program:
    // create the third qualifying permanent, then sacrifice the storied source
    // without priority or an SBA checkpoint between these instructions.
    let item = g.activation_item(p, source, 0);
    g.state.resolving = Some(Resolution {
        chosen_creature_type: None,
        target_counter_removal: None,
        linked_copy: None,
        cascade: None,
        target_card_facts: None,
        sacrificed_power: None,
        immediate_blink_return: None,
        connive: None,
        proliferate: None,
        explore: None,
        evidence: None,
        item,
        mana_payment: None,
        forage: None,
        target_library: None,
        program: &[
            Instruction::CreateTokens {
                token: Card::MshDoombotToken,
                amount: 1,
                tapped: false,
            },
            Instruction::SacrificeSource,
        ],
        pc: 0,
        target_group: 0,
        moved_count: None,
        remembered_draws: None,
        discard_after: None,
        optional_wheel: None,
        discards: [0; 2],
        selected_discards: [vec![], vec![]],
        milled: vec![],
        milled_selection: None,
        library: None,
        selection: None,
        return_attachment: None,
        search: None,
        search_result: None,
        optional_searches: None,
        choice: None,
    });
    g.run_program();
    assert!(enduring(&g, p));
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    assert_eq!(
        g.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield)
            .count(),
        2
    );
}

#[test]
fn hob_storied_current_types_and_granted_legendary_feed_the_threshold() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let artifact = put(&mut g, p, "Diamond Mare", Zone::Battlefield);
        put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let aura = put(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, aura, Some(Target::Card(artifact)));
        settle(&mut g);
        assert_eq!(g.object_view(artifact).kinds & 16, 0);
        paid(&mut g, p, "Ori, Keeper of Songs");
        assert!(
            !enduring(&g, p),
            "printed artifact type is absent in the current world"
        );
        let disenchant = put(&mut g, p, "Disenchant", Zone::Hand);
        cast(&mut g, disenchant, Some(Target::Card(aura)));
        settle(&mut g);
        assert!(
            enduring(&g, p),
            "returning artifact type completes the threshold"
        );

        let mut g = scenario(p);
        paid(&mut g, p, "Ori, Keeper of Songs");
        put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.refresh_controllers();
        assert!(!enduring(&g, p));
        let serum = put(&mut g, p, "Super-Soldier Serum", Zone::Hand);
        cast(&mut g, serum, Some(Target::Card(bear)));
        settle(&mut g);
        assert!(
            enduring(&g, p),
            "continuous legendary grant counts, not printed metadata"
        );
        assert!(ability_world::CharacteristicWorld::legendary(&g, bear));
    }
}

#[test]
fn hob_storied_continuous_control_counts_actual_controller_and_persists() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        paid(&mut g, p, "Ori, Keeper of Songs");
        put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Diamond Mare", Zone::Battlefield);
        let confiscate = put(&mut g, p, "Confiscate", Zone::Hand);
        cast(&mut g, confiscate, Some(Target::Card(enemy)));
        settle(&mut g);
        assert_eq!(g.object_view(enemy).controller, p);
        assert!(enduring(&g, p));
        assert!(!enduring(&g, p.other()));
        let disenchant = put(&mut g, p, "Disenchant", Zone::Hand);
        cast(&mut g, disenchant, Some(Target::Card(confiscate)));
        settle(&mut g);
        assert_eq!(g.object_view(enemy).controller, p.other());
        assert!(
            enduring(&g, p),
            "losing the controlled third permanent cannot remove it"
        );
    }
}
