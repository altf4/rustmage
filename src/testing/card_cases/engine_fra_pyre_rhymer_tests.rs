//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

// Keep the catalogue gated while using checked actions against authored rules.
fn pyre_fixture(g: &mut Game, player: Player, zone: Zone) -> CardId {
    let id = put(g, player, "Bear Cub", zone);
    g.state.objects[id as usize].card = Card::from_name("Pyre Rhymer // Molten Tide").unwrap();
    if zone == Zone::Battlefield {
        g.register_battlefield_sources(id);
    }
    id
}

// Draft-only front face coverage: Molten Tide remains unsupported and gated.

#[test]
fn pyre_rhymer_prowess_responds_only_to_own_noncreature_cast() {
    {
        let player = Player::P0;
        let mut g = scenario(player);
        let source = pyre_fixture(&mut g, player, Zone::Battlefield);
        let creature = put(&mut g, player, "Bear Cub", Zone::Hand);
        cast(&mut g, creature, None);
        settle(&mut g);
        assert_eq!(stats(&g, source), (3, 3));
        let spell = put(&mut g, player, "Shock", Zone::Hand);
        cast(&mut g, spell, Some(Target::Player(player.other())));
        settle(&mut g);
        assert_eq!(stats(&g, source), (4, 4));
        g.finish_cleanup();
        assert_eq!(stats(&g, source), (3, 3));
    }
}

fn grant_molten_tide(g: &mut Game, player: Player) -> CardId {
    let source = pyre_fixture(g, player, Zone::Battlefield);
    let reference = g.object_ref(source);
    g.prepare_permanent(reference);
    let copy = g
        .state
        .preparations
        .as_ref()
        .unwrap()
        .iter()
        .find(|row| row.permanent == reference)
        .unwrap()
        .spell_copy;
    g.state.players[player.index()].mana = [0, 0, 0, 1, 0, 0];
    act(g, Action::AnnounceExile { card: copy.card });
    pay(g);
    for _ in 0..96 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return source;
        }
        choose(g, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("Molten Tide resolution did not finish");
}

#[test]
fn molten_tide_bonus_is_unrestricted_when_original_mana_is_restricted() {
    {
        let player = Player::P0;
        let mut g = scenario(player);
        grant_molten_tide(&mut g, player);
        let land = put(&mut g, player, "Castle Doom", Zone::Battlefield);
        let reference = g.object_ref(land);
        let item = g.cast_preview(player, land, Zone::Battlefield, reference, 0, [0; 6]);
        g.add_basic_land_type(
            &item,
            Subject::Source,
            crate::cards::BasicLandType::Mountain,
        );
        g.state.players[player.index()].mana = [0; 6];
        let activation = g.legal_actions().into_iter().find(|a| matches!(a,
            Action::ActivateMana { card, ability: 1, production: [0,0,0,1,0,0], .. } if *card == land)).unwrap();
        act(&mut g, activation);
        assert_eq!(g.state.players[player.index()].mana[3], 2);
        let shock = put(&mut g, player, "Shock", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: shock });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(player.other()),
            },
        );
        pay(&mut g);
        assert_eq!(g.state.players[player.index()].mana[3], 1);
        assert_eq!(
            g.mana_lots(player)
                .iter()
                .find(|lot| lot.quality.restriction == ManaRestrictionView::ArtifactSpell)
                .unwrap()
                .amounts[3],
            1
        );
    }
}

#[test]
fn molten_tide_extra_mana_follows_kept_or_reversed_mana_event() {
    {
        let player = Player::P0;
        for keep in [false, true] {
            let mut g = scenario(player);
            let pyre = grant_molten_tide(&mut g, player);
            // The granting creature would otherwise remain eligible for casualty.
            g.move_object(pyre, Zone::Graveyard);
            let silver = put(
                &mut g,
                player,
                "Silverquill, the Disputant",
                Zone::Battlefield,
            );
            g.state.objects[silver as usize].growth.power = -1;
            let enemy = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
            let aura = put(
                &mut g,
                player.other(),
                "Witness Protection",
                Zone::Battlefield,
            );
            g.state.objects[aura as usize].attached_to = Some(g.object_ref(enemy));
            let prison = put(&mut g, player, "Banishing Light", Zone::Hand);
            g.state.players[player.index()].mana = [8; 6];
            super::super::big_helpers::cast(&mut g, prison, None, &[]);
            for _ in 0..40 {
                if g.state.pending != Pending::Priority {
                    break;
                }
                act(&mut g, Action::Pass);
            }
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(aura),
                },
            );
            super::super::big_helpers::settle(&mut g);
            let mountain = put(&mut g, player, "Mountain", Zone::Battlefield);
            let spell = put(&mut g, player, "Candy Grapple", Zone::Hand);
            g.state.players[player.index()].mana = [0, 0, 1, 0, 0, 0];
            act(&mut g, Action::AnnounceCast { card: spell });
            let sacrifice = g.object_ref(prison);
            act(
                &mut g,
                Action::ChooseBargain {
                    sacrifice: Some(sacrifice),
                },
            );
            act(&mut g, Action::ChooseCasualty { pay: true });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            act(&mut g, Action::TapMana { card: mountain });
            pay(&mut g);
            let clause = g
                .observe(player)
                .cast_costs
                .unwrap()
                .clauses
                .iter()
                .position(|row| row.clause == CastCostClause::Additional)
                .unwrap() as u16;
            act(&mut g, Action::ChooseCastCost { clause });
            let silver = g.object_ref(silver);
            act(&mut g, Action::ChooseAuraAttachment { object: silver });
            assert!(matches!(g.state.pending, Pending::ManaRollback { .. }));
            assert_eq!(
                g.observe(player).mana_rollback.unwrap().mountain_bonus,
                Some(1)
            );
            act(&mut g, Action::ChooseManaRollback { keep });
            assert_eq!(g.object_view(spell).zone, Zone::Hand);
            assert_eq!(g.object_view(mountain).tapped, keep);
            assert_eq!(
                g.state.players[player.index()].mana,
                [0, 0, 1, if keep { 2 } else { 0 }, 0, 0]
            );
        }
    }
}
