use super::*;

fn source(g: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    let id = put(g, owner, "Bear Cub", zone);
    g.state.objects[id as usize].card = Card::from_name(name).unwrap();
    if zone == Zone::Battlefield {
        g.register_battlefield_sources(id);
    }
    id
}

#[test]
fn scourge_artifact_creature_cast_triggers_once_even_if_countered() {
    for p in [Player::P0, Player::P1] {
        for caster in [p, p.other()] {
            let mut g = scenario(caster);
            let dragon = source(&mut g, p, "Traxos, Scourge Eternal", Zone::Battlefield);
            g.state.objects[dragon as usize].tapped = true;
            let artifact = put(&mut g, caster, "A.I.M. Synthoids", Zone::Hand);
            cast(&mut g, artifact, None);
            g.prepare_triggers();
            let triggers = g
                .state
                .stack
                .iter()
                .filter(|item| {
                    item.source.unwrap().id == dragon
                        && matches!(item.kind, StackKind::TriggeredAbility { .. })
                })
                .count();
            assert_eq!(triggers, usize::from(caster == p));
            let spell = g
                .state
                .stack
                .iter()
                .find(|item| item.source.unwrap().id == artifact && item.kind == StackKind::Spell)
                .unwrap()
                .id;
            assert!(g.counter_stack_item(spell, false));
            super::contracts::branch_contract(g.clone());
            let before = g.snapshot();
            settle(&mut g);
            assert_eq!(g.object_view(artifact).zone, Zone::Graveyard);
            assert_eq!(g.object_view(dragon).tapped, caster != p);
            g.restore(&before).unwrap();
            settle(&mut g);
            assert_eq!(g.object_view(dragon).tapped, caster != p);
        }
    }
}

#[test]
fn scourge_trigger_does_not_untap_a_new_incarnation_or_trigger_after_ability_loss() {
    for p in [Player::P0, Player::P1] {
        for suppressed in [false, true] {
            let mut g = scenario(p);
            let dragon = source(&mut g, p, "Traxos, Scourge Eternal", Zone::Battlefield);
            g.state.objects[dragon as usize].tapped = true;
            if suppressed {
                let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
                g.state.objects[aura as usize].attached_to = Some(g.object_ref(dragon));
                assert!(!g.printed_abilities_available(dragon));
            }
            let artifact = put(&mut g, p, "A.I.M. Synthoids", Zone::Hand);
            cast(&mut g, artifact, None);
            g.prepare_triggers();
            let triggers = g
                .state
                .stack
                .iter()
                .filter(|item| {
                    item.source.unwrap().id == dragon
                        && matches!(item.kind, StackKind::TriggeredAbility { .. })
                })
                .count();
            assert_eq!(triggers, usize::from(!suppressed));
            if !suppressed {
                let original = g.object_ref(dragon);
                g.move_object(dragon, Zone::Exile);
                g.move_object(dragon, Zone::Battlefield);
                assert_ne!(original, g.object_ref(dragon));
                g.state.objects[dragon as usize].tapped = true;
            }
            let spell = g
                .state
                .stack
                .iter()
                .find(|item| item.source.unwrap().id == artifact && item.kind == StackKind::Spell)
                .unwrap()
                .id;
            assert!(g.counter_stack_item(spell, false));
            settle(&mut g);
            assert!(g.object_view(dragon).tapped);
        }
    }
}

#[test]
fn academy_ignores_opponents_casts_for_discount_and_prowess() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p.other());
        let dragon = source(&mut g, p, "Traxos, Academy Guardian", Zone::Battlefield);
        let opposing = put(&mut g, p.other(), "Shock", Zone::Hand);
        cast(&mut g, opposing, Some(Target::Player(p)));
        settle(&mut g);
        assert_eq!(stats(&g, dragon), (1, 5));
        g.state.active = p;
        g.state.priority = p;
        let copy = source(&mut g, p, "Traxos, Academy Guardian", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
        reject(&mut g, Action::AnnounceCast { card: copy });
        assert_eq!(g.object_view(copy).zone, Zone::Hand);
        g.state.players[p.index()].mana = [8; 6];
        for expected in [(2, 6), (3, 7)] {
            let shock = put(&mut g, p, "Shock", Zone::Hand);
            cast(&mut g, shock, Some(Target::Player(p.other())));
            settle(&mut g);
            assert_eq!(stats(&g, dragon), expected);
        }
        super::contracts::branch_contract(g.clone());
        g.finish_cleanup();
        assert_eq!(stats(&g, dragon), (1, 5));
    }
}
