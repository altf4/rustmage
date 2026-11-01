//! Conditional announcement permissions and persistent first-creature history.
use super::*;
fn instant_window(g: &mut Game, p: Player) {
    g.state.active = p.other();
    g.state.priority = p;
    g.state.step = Step::Main1;
}
fn ready(g: &Game, card: CardId) -> bool {
    g.legal_actions().contains(&Action::AnnounceCast { card })
}
fn recruit(g: &mut Game, discard: CardId) {
    for _ in 0..24 {
        if g.legal_actions()
            .contains(&Action::Discard { card: discard })
        {
            act(g, Action::Discard { card: discard });
            settle(g);
            return;
        }
        act(g, Action::Pass);
    }
    panic!("recruit did not reach discard");
}
fn types(g: &mut Game, id: CardId, types: &'static [&'static str]) {
    // Fixture a preexisting layer-four type effect; the action under test still
    // pays the real sacrifice-mana ability and checks the actual derived subtype.
    let recipient = g.object_ref(id);
    g.state
        .roles
        .get_or_insert_with(Default::default)
        .creature_types
        .push(subtypes::CreatureTypesEffectView {
            recipient,
            source: recipient,
            types,
            additional: false,
            timestamp: 1,
        });
}
fn bolg_mana(g: &mut Game, bolg: CardId, fodder: CardId) {
    act(
        g,
        Action::AnnounceMana {
            card: bolg,
            ability: 0,
            counter_payment: Counters::default(),
            production: [0, 0, 1, 1, 0, 0],
        },
    );
    let object = g.object_ref(fodder);
    act(g, Action::ChooseAdditionalCost { object });
    pay(g);
}
#[test]
fn bard_timing_requires_current_controlled_battlefield_human_without_flash_keyword() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bard = put(&mut g, p, "Bard's Company", Zone::Hand);
        instant_window(&mut g, p);
        reject(&mut g, Action::AnnounceCast { card: bard });
        let human = put(&mut g, p.other(), "Savannah Lions", Zone::Battlefield);
        types(&mut g, human, &["Human"]);
        reject(&mut g, Action::AnnounceCast { card: bard });
        g.state.objects[human as usize].controller = p;
        assert!(ready(&g, bard));
        assert!(!g.keywords(bard).contains(Keywords::FLASH));
        g.move_object(human, Zone::Graveyard);
        reject(&mut g, Action::AnnounceCast { card: bard });
        // Its own Human subtype never permits an otherwise illegal hand cast.
        g.state.active = p;
        g.state.priority = p;
        let discard = put(&mut g, p, "Forest", Zone::Hand);
        g.state.players[p.index()].mana = [1, 1, 0, 0, 0, 2];
        cast(&mut g, bard, None);
        recruit(&mut g, discard);
        assert_eq!(stats(&g, bard), (2, 3));
        assert!(!g.keywords(bard).contains(Keywords::FLASH));
    }
}
#[test]
fn bard_paid_entry_attack_recruit_and_dynamic_other_creature_bonus() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let friend = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let discard = put(&mut g, p, "Shock", Zone::Hand);
        let drawn = put(&mut g, p, "Island", Zone::Library);
        let bard = put(&mut g, p, "Bard's Company", Zone::Hand);
        g.state.players[p.index()].mana = [1, 1, 0, 0, 0, 2];
        cast(&mut g, bard, None);
        for _ in 0..16 {
            if matches!(g.state.pending, Pending::EffectDiscard { .. }) {
                break;
            }
            act(&mut g, Action::Pass);
        }
        assert_eq!(g.object_view(drawn).zone, Zone::Hand);
        assert!(
            g.observe(p.other())
                .objects
                .iter()
                .all(|o| o.id != drawn && o.id != discard)
        );
        act(&mut g, Action::Discard { card: discard });
        settle(&mut g);
        assert_eq!(stats(&g, bard), (2, 3));
        assert_eq!(stats(&g, friend), (3, 3));
        assert_eq!(stats(&g, enemy), (2, 2));
        let soldiers: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.zone == Zone::Battlefield && o.card.index() == 5241)
            .map(|(i, _)| i as CardId)
            .collect();
        assert_eq!(soldiers.len(), 1);
        assert_eq!(stats(&g, soldiers[0]), (2, 2));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.state.objects[bard as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: bard });
        act(&mut g, Action::FinishAttackers);
        recruit(&mut g, drawn);
        assert_eq!(g.object_view(drawn).zone, Zone::Graveyard);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield && o.card.index() == 5241)
                .count(),
            1
        );
        g.move_object(bard, Zone::Graveyard);
        assert_eq!(stats(&g, friend), (2, 2));
        assert_eq!(stats(&g, soldiers[0]), (1, 1));
    }
}
#[test]
fn bard_approved_permission_survives_last_human_mana_sacrifice() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bolg = put(&mut g, p, "Bolg's Company", Zone::Battlefield);
        let human = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        types(&mut g, human, &["Human", "Goblin"]);
        let bard = put(&mut g, p, "Bard's Company", Zone::Hand);
        instant_window(&mut g, p);
        g.state.players[p.index()].mana = [1, 1, 0, 0, 0, 0];
        act(&mut g, Action::AnnounceCast { card: bard });
        assert_eq!(g.state.players[p.index()].creature_spells_cast_this_turn, 0);
        contracts::branch_contract(g.clone());
        bolg_mana(&mut g, bolg, human);
        assert_eq!(g.object_view(human).zone, Zone::Graveyard);
        contracts::branch_contract(g.clone());
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        pay(&mut g);
        assert_eq!(g.object_view(bard).zone, Zone::Stack);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.state.players[p.index()].creature_spells_cast_this_turn, 1);
    }
}
#[test]
fn radagast_first_generic_discount_timing_and_per_player_history() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Radagast of Rhosgobel", Zone::Battlefield);
        instant_window(&mut g, p);
        let giant = put(&mut g, p, "Hill Giant", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
        cast(&mut g, giant, None);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.state.players[p.index()].creature_spells_cast_this_turn, 1);
        settle(&mut g);
        let second = put(&mut g, p, "Bear Cub", Zone::Hand);
        g.state.priority = p;
        g.state.players[p.index()].mana = [8; 6];
        reject(&mut g, Action::AnnounceCast { card: second });
        assert_eq!(
            g.state.players[p.other().index()].creature_spells_cast_this_turn,
            0
        );
        // The used permission also stops discounting ordinary main-phase casts.
        g.state.active = p;
        g.state.priority = p;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
        reject(&mut g, Action::AnnounceCast { card: second });
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
        cast(&mut g, second, None);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        g.state.active = p.other();

        put(
            &mut g,
            p.other(),
            "Radagast of Rhosgobel",
            Zone::Battlefield,
        );
        g.state.priority = p.other();
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
        g.state.players[p.other().index()].mana = [0, 0, 0, 0, 1, 0];
        cast(&mut g, enemy, None);
        assert_eq!(g.state.players[p.other().index()].mana, [0; 6]);
        assert_eq!(
            g.state.players[p.other().index()].creature_spells_cast_this_turn,
            1
        );
    }
}
#[test]
fn radagast_prior_countered_cast_and_own_cast_consume_history_before_provider() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let first = put(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, first, None);
        act(&mut g, Action::Pass);
        let counter = put(&mut g, p.other(), "Essence Scatter", Zone::Hand);
        cast(&mut g, counter, Some(Target::Spell(first)));
        settle(&mut g);
        assert_eq!(g.object_view(first).zone, Zone::Graveyard);
        put(&mut g, p, "Radagast of Rhosgobel", Zone::Battlefield);
        instant_window(&mut g, p);
        let next = put(&mut g, p, "Bear Cub", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: next });
        g.begin_turn();
        g.state.pending = Pending::Priority;
        g.state.priority = p;
        g.state.step = Step::Main1;
        assert_eq!(g.state.players[p.index()].creature_spells_cast_this_turn, 0);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
        cast(&mut g, next, None);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let mut g = scenario(p);
        let rad = put(&mut g, p, "Radagast of Rhosgobel", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 2, 2];
        cast(&mut g, rad, None);
        settle(&mut g);
        assert_eq!(stats(&g, rad), (2, 5));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        instant_window(&mut g, p);
        let next = put(&mut g, p, "Bear Cub", Zone::Hand);
        g.state.players[p.index()].mana = [8; 6];
        reject(&mut g, Action::AnnounceCast { card: next });
    }
}
#[test]
fn radagast_locked_cost_and_permission_survive_provider_sacrifice_for_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let rad = put(&mut g, p, "Radagast of Rhosgobel", Zone::Battlefield);
        types(&mut g, rad, &["Goblin"]);
        let bolg = put(&mut g, p, "Bolg's Company", Zone::Battlefield);
        instant_window(&mut g, p);
        let giant = put(&mut g, p, "Hill Giant", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        act(&mut g, Action::AnnounceCast { card: giant });
        bolg_mana(&mut g, bolg, rad);
        contracts::branch_contract(g.clone());
        pay(&mut g);
        assert_eq!(g.object_view(rad).zone, Zone::Graveyard);
        assert_eq!(g.object_view(giant).zone, Zone::Stack);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn radagast_adventure_face_nonhand_creature_and_copied_card_cast() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Radagast of Rhosgobel", Zone::Battlefield);
        let goblin = put(
            &mut g,
            p,
            "Great Ugly-Looking Goblin // Clap! Snap!",
            Zone::Hand,
        );
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 1];
        act(&mut g, Action::AnnounceAdventure { card: goblin });
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].creature_spells_cast_this_turn, 0);
        assert_eq!(g.state.players[p.index()].instant_sorcery_cast_this_turn, 1);
        settle(&mut g);
        instant_window(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 3];
        act(&mut g, Action::AnnounceExile { card: goblin });
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].creature_spells_cast_this_turn, 1);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        for origin in [Zone::Graveyard, Zone::Exile] {
            let mut g = scenario(p);
            put(&mut g, p, "Radagast of Rhosgobel", Zone::Battlefield);
            let bear = if origin == Zone::Exile {
                g.allocate_card_copy(Card::from_name("Bear Cub").unwrap(), p)
                    .card
            } else {
                put(&mut g, p, "Bear Cub", origin)
            };
            let object = g.object_ref(bear);
            if origin == Zone::Exile {
                g.grant_exile_play_permission(p, object, false);
            } else {
                g.grant_graveyard_permission(p, object, GraveyardCastKind::Play);
            }
            instant_window(&mut g, p);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
            if origin == Zone::Exile {
                act(&mut g, Action::AnnounceExile { card: bear });
                pay(&mut g);
            } else {
                cast(&mut g, bear, None);
            }
            assert_eq!(g.state.players[p.index()].creature_spells_cast_this_turn, 1);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            contracts::branch_contract(g.clone());
        }
    }
}
#[test]
fn radagast_noncast_copy_does_not_increment_creature_history() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Radagast of Rhosgobel", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, bear, None);
        let sparks = put(&mut g, p, "Choreographed Sparks", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: sparks });
        act(&mut g, Action::ChooseMode { mode: 1 });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Spell(bear),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].creature_spells_cast_this_turn, 1);
        assert_eq!(g.state.players[p.index()].instant_sorcery_cast_this_turn, 1);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield
                    && o.card == Card::from_name("Bear Cub").unwrap())
                .count(),
            2
        );
    }
}
