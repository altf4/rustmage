//! Focused printed outcomes; shared engine invariants live in core tests.
//! Partial battle foundation: targeting and damage to defense counters.
//! Defeat, protector and combat behavior remain separate unsupported work.
use super::*;

#[test]
fn any_target_accepts_battle_and_damage_removes_defense() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let battle = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let mut recipe = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
        recipe.overrides = Some(CopyOverrides {
            kinds: Some(crate::cards::BATTLE_KIND),
            ..CopyOverrides::NONE
        });
        g.apply_temporary_copy(g.object_ref(battle), recipe, CopyDeadline::EndOfTurn);
        assert_eq!(g.current_kinds(battle), crate::cards::BATTLE_KIND);
        g.add_counter(battle, CounterKind::Defense, 5);
        let spell = put(&mut g, p, "Shock", Zone::Hand);
        big_helpers::checked(&mut g, Action::AnnounceCast { card: spell });
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(battle),
            },
        );
        pay(&mut g);
        big_helpers::settle(&mut g);
        assert_eq!(g.counter_count(battle, CounterKind::Defense), 3);
        assert_eq!(g.object_view(battle).damage, 0);
    }
}

fn real_siege(g: &mut Game, p: Player) -> CardId {
    let siege = put(g, p, "@fixture/siege-front", Zone::Hand);
    big_helpers::cast(g, siege, None, &[]);
    big_helpers::settle(g);
    assert_eq!(g.counter_count(siege, CounterKind::Defense), 5);
    assert_eq!(g.battle_protector(g.object_ref(siege)), Some(p.other()));
    assert_eq!(g.object_view(siege).battle_protector, Some(p.other()));
    siege
}

#[test]
fn siege_last_counter_queues_defeat_then_offers_only_transformed_cast() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let siege = real_siege(&mut g, p);
        let old = g.object_ref(siege);
        g.pay_counters(siege, &Counters::of(CounterKind::Defense, 5));
        assert_eq!(g.state.pending_triggers.len(), 1);
        g.state_based_actions();
        assert_eq!(g.object_view(siege).zone, Zone::Battlefield);
        g.prepare_triggers();
        for _ in 0..40 {
            if matches!(g.state.pending, Pending::ResolutionCasts { .. }) {
                break;
            }
            let actions = g.legal_actions();
            let action = actions
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                .unwrap();
            big_helpers::checked(&mut g, action);
        }
        assert!(matches!(g.state.pending, Pending::ResolutionCasts { .. }));
        assert_eq!(g.object_view(siege).zone, Zone::Exile);
        let exile = g.object_ref(siege);
        assert_ne!(old, exile);
        let back = Action::CastExiledDuringResolution {
            object: exile,
            face: CastFace::Back,
        };
        assert!(g.legal_actions().contains(&back));
        assert!(
            !g.legal_actions()
                .contains(&Action::CastExiledDuringResolution {
                    object: exile,
                    face: CastFace::Normal
                })
        );
        big_helpers::checked(&mut g, back);
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        big_helpers::checked(&mut g, pay);
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(siege).zone, Zone::Battlefield);
        assert_eq!(g.object_view(siege).card.name(), "@fixture/siege-back");
        assert_eq!(g.stats(siege).power, 3);
    }
}

#[test]
fn greatwurm_cast_removes_last_defense_and_defeat_waits_until_payment_finishes() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let siege = real_siege(&mut g, p);
        let object = g.object_ref(siege);
        let mut recipe = CopyRecipe::new(Card::from_name("@fixture/siege-front").unwrap());
        recipe.overrides = Some(CopyOverrides {
            kinds: Some(1 | crate::cards::BATTLE_KIND),
            ..CopyOverrides::NONE
        });
        g.apply_temporary_copy(object, recipe, CopyDeadline::EndOfTurn);
        // The artificial type override supplies no creature base stats.
        g.state.objects[siege as usize].growth = Stats {
            power: 2,
            toughness: 2,
        };
        g.add_counter(siege, CounterKind::Defense, 1);
        let wurm = put(&mut g, p, "Quilled Greatwurm", Zone::Graveyard);
        big_helpers::checked(&mut g, Action::AnnounceCounterGraveyard { card: wurm });
        for _ in 0..6 {
            big_helpers::checked(
                &mut g,
                Action::ChooseCreatureCounterCost {
                    object,
                    counter: CounterKind::Defense,
                },
            );
        }
        assert_eq!(g.counter_count(siege, CounterKind::Defense), 6);
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        big_helpers::checked(&mut g, pay);
        assert_eq!(g.counter_count(siege, CounterKind::Defense), 0);
        assert_eq!(g.object_view(siege).zone, Zone::Battlefield);
        assert!(
            g.state
                .pending_triggers
                .iter()
                .chain(&g.state.ordering_triggers)
                .chain(&g.state.stack)
                .any(|item| item.kind == StackKind::SiegeDefeat
                    && item.source.unwrap().object == object)
        );
    }
}

#[test]
fn mandatory_attacker_can_attack_planeswalker_when_player_is_taxed() {
    let p = Player::P0;
    let mut g = scenario(p);
    let attacker = put(&mut g, p, "Juggernaut", Zone::Battlefield);
    let dain = put(
        &mut g,
        p.other(),
        "Dáin, Lord of the Iron Hills",
        Zone::Battlefield,
    );
    g.state.players[p.other().index()].enduring_story = true;
    assert_eq!(g.attack_controller_tax(p.other()), 1);
    let walker = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let mut recipe = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
    recipe.overrides = Some(CopyOverrides {
        kinds: Some(64),
        ..CopyOverrides::NONE
    });
    g.apply_temporary_copy(g.object_ref(walker), recipe, CopyDeadline::EndOfTurn);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    assert!(g.can_declare_attacker(p, attacker));
    assert!(!g.attack_declaration_complete(p));
    let _ = dain;
}
