//! Real LCI spells exercise resolution cohorts, exact targets and turn history.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn lci(game: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, owner, Card::from_name(name).unwrap(), zone)
}

fn next_turn(game: &mut Game) {
    let turn = game.state.turn;
    for _ in 0..30 {
        if game.state.turn != turn {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass | Action::FinishAttackers | Action::FinishBlockers
            )
        });
    }
    panic!("next turn not reached");
}

#[test]
fn battle_glyph_counters_before_fighting_and_keeps_its_legal_friendly_target() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let other = player.other();
        let own = put(&mut game, player, "Bear Cub", Zone::Hand);
        let enemy = put(&mut game, other, "Grizzly Bears", Zone::Battlefield);
        let old = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let spell = lci(&mut game, player, "Malamet Battle Glyph", Zone::Hand);
        next_turn(&mut game);
        next_turn(&mut game);
        for _ in 0..8 {
            if game.state.step == Step::Main1 {
                break;
            }
            act(&mut game, Action::Pass);
        }
        assert_eq!(game.state.active, player);
        assert_eq!(game.state.step, Step::Main1);
        assert!(!game.object_view(old).entered_this_turn);
        game.state.players[player.index()].mana = [8; 6];
        cast(&mut game, own, None);
        settle(&mut game);

        let mut fresh = game.clone();
        announce(&mut fresh, spell, &[Target::Card(own), Target::Card(enemy)]);
        let saved = fresh.snapshot();
        settle(&mut fresh);
        assert_eq!(stats(&fresh, own), (3, 3));
        assert_eq!(fresh.object_view(own).damage, 2);
        assert_eq!(fresh.object_view(enemy).zone, Zone::Graveyard);

        fresh.restore(&saved).unwrap();
        let removal = put(&mut fresh, player, "Shock", Zone::Hand);
        cast(&mut fresh, removal, Some(Target::Card(enemy)));
        settle(&mut fresh);
        assert_eq!(stats(&fresh, own), (3, 3));
        assert_eq!(fresh.object_view(own).damage, 0);

        announce(&mut game, spell, &[Target::Card(old), Target::Card(enemy)]);
        settle(&mut game);
        assert_eq!(stats(&game, old), (3, 3));
        assert_eq!(game.object_view(old).damage, 2);
        assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
    }
}

#[test]
fn eclipse_uses_the_resolution_cohort_and_replaces_later_opposing_deaths() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let other = player.other();
        let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let foe = put(&mut game, other, "Grizzly Bears", Zone::Battlefield);
        let survives = put(&mut game, other, "Hill Giant", Zone::Battlefield);
        let late = put(&mut game, other, "Bear Cub", Zone::Hand);
        let eclipse = lci(&mut game, player, "Malicious Eclipse", Zone::Hand);
        cast(&mut game, eclipse, None);
        settle(&mut game);
        assert_eq!(game.object_view(own).zone, Zone::Graveyard);
        assert_eq!(game.object_view(foe).zone, Zone::Exile);
        assert_eq!(stats(&game, survives), (1, 1));
        assert_eq!(game.object_view(eclipse).zone, Zone::Graveyard);

        let movement = game.entry_move(late, Zone::Battlefield, false);
        game.stage_entries(vec![movement], None);
        game.advance_entries();
        assert_eq!(stats(&game, late), (2, 2));
        let shock = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, shock, Some(Target::Card(late)));
        settle(&mut game);
        assert_eq!(game.object_view(late).zone, Zone::Exile);

        next_turn(&mut game);
        assert_eq!(stats(&game, survives), (3, 3));
        game.state.players[other.index()].mana = [8; 6];
        let kill = put(&mut game, other, "Murder", Zone::Hand);
        cast(&mut game, kill, Some(Target::Card(survives)));
        settle(&mut game);
        assert_eq!(game.object_view(survives).zone, Zone::Graveyard);
    }
}

#[test]
fn petrify_suppresses_combat_and_mana_for_either_enchant_type() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let elf = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
        let treasure = game.state.objects.len() as CardId;
        game.create_tokens(player, Card(5302), 1, false);
        game.advance_entries();
        for recipient in [elf, treasure] {
            let mana = game.legal_actions().into_iter().find(|action| {
                matches!(action, Action::ActivateMana { card, .. } | Action::TapMana { card } if *card == recipient)
            }).unwrap();
            let aura = lci(&mut game, player, "Petrify", Zone::Hand);
            cast(&mut game, aura, Some(Target::Card(recipient)));
            settle(&mut game);
            assert_eq!(
                game.object_view(aura).attached_to,
                Some(game.object_ref(recipient))
            );
            reject(&mut game, mana.clone());
            assert!(game.combat_restricted(recipient, true));
            assert!(game.combat_restricted(recipient, false));

            let mut attack = game.clone();
            if recipient == elf {
                attack.state.step = Step::Attackers;
                attack.state.pending = Pending::Attackers;
                reject(&mut attack, Action::Attack { card: elf });
            }

            game.move_object(aura, Zone::Graveyard);
            assert!(game.legal_actions().contains(&mana));
            assert!(!game.combat_restricted(recipient, true));
        }
    }
}

#[test]
fn relics_roar_adds_types_retains_subtypes_and_expires() {
    let player = Player::P0;
    let mut game = scenario(player);
    let elf = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    let boots = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    for (target, original_subtype) in [(elf, "Elf"), (boots, "Equipment")] {
        let spell = lci(&mut game, player, "Relic's Roar", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(target)));
        settle(&mut game);
        assert_eq!(game.object_view(target).kinds & 17, 17);
        assert_eq!(stats(&game, target), (4, 3));
        assert!(game.has_subtype(target, "Dinosaur"));
        assert!(game.has_subtype(target, original_subtype));
    }
    let mana = Action::TapMana { card: elf };
    assert!(game.legal_actions().contains(&mana));
    next_turn(&mut game);
    assert_eq!(game.object_view(elf).kinds & 17, 1);
    assert_eq!(game.object_view(boots).kinds & 17, 16);
    assert_eq!(stats(&game, elf), (1, 1));
    assert!(!game.has_subtype(boots, "Dinosaur"));
}

#[test]
fn scytheclaw_hits_the_off_turn_caster_including_its_own_controller() {
    for active in [Player::P0, Player::P1] {
        let off = active.other();
        let mut game = scenario(active);
        let raptor = lci(&mut game, off, "Scytheclaw Raptor", Zone::Battlefield);
        let main_shock = put(&mut game, active, "Shock", Zone::Hand);
        let reply = put(&mut game, off, "Shock", Zone::Hand);
        assert_eq!(stats(&game, raptor), (4, 3));
        cast(&mut game, main_shock, Some(Target::Player(off)));
        settle(&mut game);
        assert_eq!(game.state.players[active.index()].life, STARTING_LIFE);
        assert_eq!(game.state.players[off.index()].life, STARTING_LIFE - 2);
        act(&mut game, Action::Pass);
        cast(&mut game, reply, Some(Target::Player(active)));
        settle(&mut game);
        assert_eq!(game.state.players[active.index()].life, STARTING_LIFE - 2);
        assert_eq!(game.state.players[off.index()].life, STARTING_LIFE - 6);
    }
}
