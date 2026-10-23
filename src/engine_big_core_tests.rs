//! Paid interactions for the Big Score's mana, lock, and zone-replacement cards.
use super::ability_timeline_tests::{cast, checked, next_turn, settle, setup};
use super::tests::spawn;
use super::*;
use crate::cards::ChoiceColor;

fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    let id = spawn(g, p, card, zone);
    g.register_zone_rule_card(card);
    id
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = fixture(g, p, name, Zone::Hand);
    cast(g, id, None, targets);
    id
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
fn optional(g: &mut Game, accept: bool) {
    for _ in 0..40 {
        if matches!(g.state.pending, Pending::EffectOptional { .. }) {
            checked(g, Action::ChooseOptional { accept });
            return;
        }
        let action = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| !matches!(a, Action::Concede))
                .unwrap()
        };
        checked(g, action);
    }
    panic!("missing optional choice");
}
fn activation(g: &mut Game, id: CardId, index: u16) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: index,
            counter_payment: Counters::default(),
        },
    );
    for _ in 0..20 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap();
        checked(g, action);
    }
    panic!("unfinished activation");
}

#[test]
fn big_cornucopia_decline_colored_amount_once_resolution_and_reentry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let cornucopia = paid(&mut g, p, "Ancient Cornucopia", &[]);
        settle(&mut g);
        let life = g.state.players[p.index()].life;
        paid(&mut g, p, "Llanowar Elves", &[]);
        optional(&mut g, false);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life);
        paid(&mut g, p, "Kutzil, Malamet Exemplar", &[]);
        optional(&mut g, true);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 2);
        paid(&mut g, p, "Savannah Lions", &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 2);
        assert_eq!(
            g.state
                .triggered_this_turn
                .iter()
                .filter(|row| row.source == g.object_ref(cornucopia))
                .count(),
            1
        );
        let action = g.legal_actions().into_iter().find(|a| matches!(a, Action::ActivateMana { card, production: [0,0,0,0,1,0], .. } if *card == cornucopia)).unwrap();
        checked(&mut g, action);
        let old = g.object_ref(cornucopia);
        g.move_object(cornucopia, Zone::Hand);
        cast(&mut g, cornucopia, None, &[]);
        settle(&mut g);
        assert_ne!(g.object_ref(cornucopia), old);
        paid(&mut g, p, "Llanowar Elves", &[]);
        optional(&mut g, true);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 3);
    }
}

#[test]
fn big_rest_in_peace_entry_every_zone_tokens_and_own_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let old = fixture(&mut g, p, "Savannah Lions", Zone::Graveyard);
        let enemy_old = fixture(&mut g, p.other(), "Llanowar Elves", Zone::Graveyard);
        let rip = paid(&mut g, p, "Rest in Peace", &[]);
        settle(&mut g);
        assert_eq!(g.object_view(old).zone, Zone::Exile);
        assert_eq!(g.object_view(enemy_old).zone, Zone::Exile);
        let victim = fixture(&mut g, p.other(), "Savannah Lions", Zone::Battlefield);
        let murder = paid(&mut g, p, "Murder", &[Target::Card(victim)]);
        settle(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Exile);
        assert_eq!(g.object_view(murder).zone, Zone::Exile);
        assert_eq!(g.state.creatures_died_this_turn, 0);
        for zone in [Zone::Hand, Zone::Library, Zone::Exile] {
            let id = fixture(&mut g, p.other(), "Savannah Lions", zone);
            let old_ref = g.object_ref(id);
            if zone == Zone::Exile {
                g.grant_exile_play_permission(p.other(), old_ref, true);
            }
            g.move_object(id, Zone::Graveyard);
            assert_eq!(g.object_view(id).zone, Zone::Exile);
            assert_ne!(g.object_ref(id), old_ref);
            assert!(!g.can_play_exiled(p.other(), id));
        }
        let token = Card::catalogue()
            .find(|card| card.token_info().is_some_and(|i| i.name == "Sliver Token"))
            .unwrap();
        g.create_tokens(p, token, 1, false);
        let id = g
            .state
            .objects
            .iter()
            .rposition(|o| o.card == token && o.zone == Zone::Battlefield)
            .unwrap() as CardId;
        let before = g.object_view(id);
        let frozen = g.freeze_zone_rules(&before, Zone::Graveyard, None);
        assert_eq!(
            g.plan_zone_move(&before, Zone::Graveyard, frozen)
                .destination,
            Zone::Exile
        );
        g.move_object(id, Zone::Graveyard);
        assert_eq!(g.state.creatures_died_this_turn, 0);
        let disenchant = paid(&mut g, p, "Disenchant", &[Target::Card(rip)]);
        settle(&mut g);
        assert_eq!(g.object_view(rip).zone, Zone::Exile);
        assert_eq!(g.object_view(disenchant).zone, Zone::Graveyard);
    }
}

#[test]
fn big_abolisher_stops_opposing_spell_and_mana_abilities_allows_lands_and_releases() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let abolisher = paid(&mut g, p, "Grand Abolisher", &[]);
        settle(&mut g);
        let q = p.other();
        let elf = fixture(&mut g, q, "Llanowar Elves", Zone::Battlefield);
        let artifact = fixture(&mut g, q, "Ancient Cornucopia", Zone::Battlefield);
        let forest = fixture(&mut g, q, "Forest", Zone::Battlefield);
        let spell = fixture(&mut g, q, "Giant Growth", Zone::Hand);
        g.state.players[q.index()].mana = [0; 6];
        checked(&mut g, Action::Pass);
        reject(&mut g, Action::AnnounceCast { card: spell });
        reject(&mut g, Action::TapMana { card: elf });
        assert!(!g.legal_actions().iter().any(|a| matches!(a, Action::ActivateMana { card, .. } | Action::AnnounceMana { card, .. } if *card == artifact)));
        checked(&mut g, Action::TapMana { card: forest });
        assert!(!g.can_fund(funding::FundingGoal {
            actor: q,
            required: [0, 0, 0, 0, 2, 0],
            generic: 0,
            reserve: None,
            source: None
        }));
        g.move_object(abolisher, Zone::Exile);
        assert!(g.legal_actions().contains(&Action::TapMana { card: elf }));
        assert!(g.can_fund(funding::FundingGoal {
            actor: q,
            required: [0, 0, 0, 0, 2, 0],
            generic: 0,
            reserve: None,
            source: None
        }));
        checked(&mut g, Action::TapMana { card: elf });
    }
}

#[test]
fn big_vista_paid_entry_chosen_color_and_distinct_monocolor_mana_in_funding() {
    for p in [Player::P0, Player::P1] {
        for chosen in ChoiceColor::ALL {
            let mut g = setup(p);
            let vista = fixture(&mut g, p, "Tarnation Vista", Zone::Hand);
            checked(&mut g, Action::PlayLand { card: vista });
            checked(
                &mut g,
                Action::ChooseEntryValue {
                    value: EntryChoiceValue::Color(chosen),
                },
            );
            assert!(g.object_view(vista).tapped);
            assert_eq!(
                g.object_view(vista).entry_choice,
                Some(EntryChoiceValue::Color(chosen))
            );
            g.state.objects[vista as usize].tapped = false;
            let mut production = [0; 6];
            production[chosen.index()] = 1;
            let a = g.legal_actions().into_iter().find(|a| matches!(a, Action::ActivateMana { card, ability: 0, production: got, .. } if *card == vista && *got == production)).unwrap();
            checked(&mut g, a);
            g.state.objects[vista as usize].tapped = false;
            fixture(&mut g, p, "Savannah Lions", Zone::Battlefield);
            fixture(&mut g, p, "Savannah Lions", Zone::Battlefield);
            fixture(&mut g, p, "Memory Vessel", Zone::Battlefield);
            fixture(&mut g, p, "Kutzil, Malamet Exemplar", Zone::Battlefield);
            fixture(&mut g, p.other(), "Ancient Cornucopia", Zone::Battlefield);
            assert_eq!(
                ManaProduction::MonocoloredPermanentColors.in_world(&g, vista),
                ManaProduction::Fixed([1, 0, 0, 1, 0, 0])
            );
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
            assert!(g.can_fund(funding::FundingGoal {
                actor: p,
                required: [1, 0, 0, 1, 0, 0],
                generic: 0,
                reserve: None,
                source: None
            }));
            let before = g.state.players[p.index()].mana;
            let action = g.legal_actions().into_iter().find(|a| matches!(a, Action::AnnounceMana { card, ability: 1, .. } if *card == vista)).unwrap();
            checked(&mut g, action);
            for _ in 0..20 {
                if g.state.announcing.is_none() {
                    break;
                }
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                    .unwrap();
                checked(&mut g, a);
            }
            assert!(g.object_view(vista).tapped);
            assert_eq!(
                g.state.players[p.index()].mana.iter().sum::<u32>(),
                before.iter().sum::<u32>() + 1
            );
        }
    }
}

#[test]
fn big_memory_vessel_exile_cost_separate_owners_hand_lock_and_exact_deadline() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let vessel = paid(&mut g, p, "Memory Vessel", &[]);
        settle(&mut g);
        let hand = fixture(&mut g, p, "Savannah Lions", Zone::Hand);
        let hand_land = fixture(&mut g, p, "Plains", Zone::Hand);
        let q_hand = fixture(&mut g, p.other(), "Llanowar Elves", Zone::Hand);
        let mut tops = Vec::new();
        for owner in [p, p.other()] {
            for index in 0..7 {
                let name = if index == 0 {
                    "Plains"
                } else {
                    "Savannah Lions"
                };
                tops.push((owner, fixture(&mut g, owner, name, Zone::Library)));
            }
        }
        activation(&mut g, vessel, 0);
        assert_eq!(
            g.object_view(vessel).zone,
            Zone::Exile,
            "source exiled as cost"
        );
        settle(&mut g);
        assert!(g.hand_play_prohibited());
        reject(&mut g, Action::AnnounceCast { card: hand });
        reject(&mut g, Action::PlayLand { card: hand_land });
        for &(owner, id) in &tops {
            assert_eq!(g.object_view(id).zone, Zone::Exile);
            assert!(g.can_play_exiled(owner, id));
            assert!(!g.can_play_exiled(owner.other(), id));
        }
        let land = tops
            .iter()
            .find(|(owner, id)| *owner == p && g.state.objects[*id as usize].card.is_land())
            .unwrap()
            .1;
        checked(&mut g, Action::PlayLand { card: land });
        assert_eq!(g.object_view(land).zone, Zone::Battlefield);
        let played = tops
            .iter()
            .find(|(owner, id)| *owner == p && !g.state.objects[*id as usize].card.is_land())
            .unwrap()
            .1;
        checked(&mut g, Action::AnnounceExile { card: played });
        for _ in 0..20 {
            if g.state.announcing.is_none() {
                break;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                .unwrap();
            checked(&mut g, a);
        }
        settle(&mut g);
        assert_eq!(g.object_view(played).zone, Zone::Battlefield);
        next_turn(&mut g);
        assert_eq!(g.state.active, p.other());
        assert!(g.hand_play_prohibited());
        // Advance to the opponent's main phase through public actions.
        for _ in 0..20 {
            if g.state.step == Step::Main1 && g.actor() == Some(p.other()) {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        reject(&mut g, Action::AnnounceCast { card: q_hand });
        next_turn(&mut g);
        assert_eq!(g.state.active, p);
        assert!(!g.hand_play_prohibited());
        for &(owner, id) in &tops {
            if id != played {
                assert!(!g.can_play_exiled(owner, id));
            }
        }
        assert!(g.state.hand_play_bans.is_none());
    }
}

#[test]
fn big_cornucopia_two_waiting_triggers_only_one_accepted_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Ancient Cornucopia", &[]);
        settle(&mut g);
        let elf = fixture(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let life = g.state.players[p.index()].life;
        paid(&mut g, p, "Giant Growth", &[Target::Card(elf)]);
        for _ in 0..20 {
            if g.state.pending == Pending::Priority
                && g.state
                    .stack
                    .iter()
                    .any(|item| matches!(item.kind, StackKind::TriggeredAbility { .. }))
            {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| !matches!(a, Action::Concede))
                .unwrap();
            checked(&mut g, action);
        }
        paid(&mut g, p, "Giant Growth", &[Target::Card(elf)]);
        optional(&mut g, true);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 1);
    }
}

#[test]
fn big_vista_mana_queries_use_owned_speculative_departures_without_touching_parent() {
    struct World<'a> {
        game: &'a Game,
        departed: CardId,
        object: Object,
    }
    impl ability_world::CharacteristicWorld for World<'_> {
        fn game(&self) -> &Game {
            self.game
        }
        fn object(&self, id: CardId) -> &Object {
            if id == self.departed {
                &self.object
            } else {
                &self.game.state.objects[id as usize]
            }
        }
        fn life(&self, player: Player) -> i128 {
            self.game.state.players[player.index()].life
        }
        fn gained_life(&self, player: Player) -> bool {
            self.game.state.players[player.index()].life_gained_this_turn > 0
        }
        fn attacking(&self, _: CardId) -> bool {
            false
        }
    }
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let vista = fixture(&mut g, p, "Tarnation Vista", Zone::Battlefield);
        let red = fixture(&mut g, p, "Memory Vessel", Zone::Battlefield);
        fixture(&mut g, p, "Savannah Lions", Zone::Battlefield);
        let before = g.snapshot();
        let mut object = g.state.objects[red as usize].clone();
        object.zone = Zone::Graveyard;
        let world = World {
            game: &g,
            departed: red,
            object,
        };
        assert_eq!(
            ManaProduction::MonocoloredPermanentColors.in_world(&world, vista),
            ManaProduction::Fixed([1, 0, 0, 0, 0, 0])
        );
        assert_eq!(
            ManaProduction::MonocoloredPermanentColors.in_world(&g, vista),
            ManaProduction::Fixed([1, 0, 0, 1, 0, 0])
        );
        assert_eq!(g.snapshot(), before);
    }
}

#[test]
fn big_rest_in_peace_freezes_simultaneous_departures_and_competing_shuffle() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let rip = paid(&mut g, p, "Rest in Peace", &[]);
        settle(&mut g);
        let victim = fixture(&mut g, p, "Savannah Lions", Zone::Battlefield);
        let events = g.move_objects_raw(&[rip, victim], Zone::Graveyard);
        g.finish_zone_events(&events, &[]);
        assert!(events.iter().all(|event| event.destination == Zone::Exile));
        assert_eq!(g.state.creatures_died_this_turn, 0);
        let rip2 = fixture(&mut g, p, "Rest in Peace", Zone::Battlefield);
        let colossus = fixture(&mut g, p, "Darksteel Colossus", Zone::Battlefield);
        let before = g.object_view(colossus);
        assert!(
            g.freeze_zone_rules(&before, Zone::Graveyard, None)
                .shuffle_self
        );
        assert!(
            g.freeze_zone_rules(&before, Zone::Graveyard, None)
                .exile_spell
        );
        g.move_object(rip2, Zone::Exile);
        assert!(
            !g.freeze_zone_rules(&before, Zone::Graveyard, None)
                .exile_spell
        );
    }
}
