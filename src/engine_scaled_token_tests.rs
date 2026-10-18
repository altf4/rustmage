use super::tests::{act, cast, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn funded(g: &mut Game, name: &str) -> CardId {
    let id = named(g, Player::P0, name, Zone::Hand);
    let (mut mana, generic) = g.state.objects[id as usize].card.cost();
    mana[5] += generic;
    g.state.players[0].mana = mana;
    id
}
fn tokens(g: &Game, p: Player, subtype: &str) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| {
            o.owner == p
                && o.zone == Zone::Battlefield
                && o.card.is_token()
                && o.card.has_subtype(subtype)
        })
        .map(|(i, _)| i as CardId)
        .collect()
}
fn replay(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    let p = g.actor().unwrap();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let after = g.snapshot();
    assert_eq!(fork.snapshot(), after);
    g.restore(&before).unwrap();
    assert_eq!(g.apply_index(p, rev, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), after);
}
fn passes(g: &mut Game) {
    replay(g, Action::Pass);
    replay(g, Action::Pass);
}
fn settle(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..2000 {
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
    panic!("did not settle");
}
fn activate(g: &mut Game, id: CardId, ability: u16, target: Option<Target>) {
    let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate{card,ability:i,target:t,..}if *card==id&&*i==ability&&*t==target)).unwrap();
    replay(g, a);
}
fn factory(subtype: &str) -> Card {
    Card::catalogue()
        .find(|c| c.is_token() && c.has_subtype(subtype))
        .unwrap()
}

#[test]
fn krenko_counts_live_owned_goblins_once_including_self_and_changeling() {
    let mut g = scenario();
    let k = named(&mut g, Player::P0, "Krenko, Mob Boss", Zone::Battlefield);
    g.create_tokens(Player::P0, factory("Goblin"), 2, false);
    g.create_tokens(Player::P1, factory("Goblin"), 1, false);
    named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
    activate(&mut g, k, 0, None);
    let old = tokens(&g, Player::P0, "Goblin");
    g.move_object(old[0], Zone::Hand);
    passes(&mut g);
    assert_eq!(tokens(&g, Player::P0, "Goblin").len(), 4);
    assert_eq!(tokens(&g, Player::P1, "Goblin").len(), 1);
    assert!(g.object_view(k).tapped);
    assert!(
        tokens(&g, Player::P0, "Goblin")
            .iter()
            .all(|&id| g.object_view(id).summoning_sick)
    );
}

#[test]
fn absent_krenko_and_no_goblins_make_zero_tokens_without_reusing_departed_source() {
    let mut g = scenario();
    let k = named(&mut g, Player::P0, "Krenko, Mob Boss", Zone::Battlefield);
    activate(&mut g, k, 0, None);
    g.move_object(k, Zone::Hand);
    passes(&mut g);
    assert!(tokens(&g, Player::P0, "Goblin").is_empty());
    assert_eq!(g.object_view(k).zone, Zone::Hand);
}

#[test]
fn ajani_all_loyalty_bodies_and_life_quantity_resolve_after_source_has_died() {
    let mut g = scenario();
    let a = named(&mut g, Player::P0, "Ajani, Caller of the Pride", Zone::Hand);
    g.move_object(a, Zone::Battlefield);
    assert_eq!(g.object_view(a).counters.loyalty, 4);
    activate(&mut g, a, 0, None);
    passes(&mut g);
    assert_eq!(g.object_view(a).counters.loyalty, 5);
    g.begin_turn();
    g.state.step = Step::Main1;
    g.give_priority();
    let b = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    activate(&mut g, a, 1, Some(Target::Card(b)));
    passes(&mut g);
    assert!(
        g.keywords(b)
            .contains(Keywords::FLYING.union(Keywords::DOUBLE_STRIKE))
    );
    g.begin_turn();
    g.state.step = Step::Main1;
    g.give_priority();
    g.state.objects[a as usize].counters.loyalty = 8;
    activate(&mut g, a, 2, None);
    assert_eq!(g.object_view(a).zone, Zone::Graveyard);
    // The effect uses current life when it resolves, not life at activation.
    let shock = funded(&mut g, "Shock");
    cast(&mut g, shock, Some(Target::Player(Player::P0)));
    passes(&mut g);
    assert_eq!(g.state.players[0].life, 18);
    passes(&mut g);
    let cats = tokens(&g, Player::P0, "Cat");
    assert_eq!(cats.len(), 18);
    for id in cats {
        let c = g.object_view(id);
        assert_eq!((c.power, c.toughness), (2, 2));
        assert_eq!(c.token.unwrap().colors, 1);
    }
}

#[test]
fn large_life_token_batch_crosses_u16_without_truncation_or_repeated_arena_scans() {
    let mut g = scenario();
    let a = named(
        &mut g,
        Player::P0,
        "Ajani, Caller of the Pride",
        Zone::Battlefield,
    );
    g.state.objects[a as usize].counters.loyalty = 8;
    g.state.players[0].life = 65_536;
    activate(&mut g, a, 2, None);
    passes(&mut g);
    assert_eq!(tokens(&g, Player::P0, "Cat").len(), 65_536);
}

#[test]
fn gather_counts_plains_subtypes_then_owned_scry_does_not_repeat_creation() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Plains", Zone::Battlefield);
    named(&mut g, Player::P0, "Temple Garden", Zone::Battlefield);
    named(&mut g, Player::P1, "Plains", Zone::Battlefield);
    named(&mut g, Player::P0, "Plains", Zone::Graveyard);
    let spell = funded(&mut g, "Gather the White Lotus");
    cast(&mut g, spell, None);
    passes(&mut g);
    assert_eq!(tokens(&g, Player::P0, "Ally").len(), 2);
    assert!(matches!(g.state.pending, Pending::EffectLibrary { .. }));
    assert_eq!(
        g.observe(Player::P0)
            .library_choice
            .unwrap()
            .remaining
            .len(),
        2
    );
    assert!(g.observe(Player::P1).library_choice.is_none());
    settle(&mut g);
    assert_eq!(tokens(&g, Player::P0, "Ally").len(), 2);
}

#[test]
fn bounty_creates_one_real_immediately_usable_treasure_for_each_owned_land() {
    let mut g = scenario();
    for _ in 0..4 {
        named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    }
    named(&mut g, Player::P1, "Forest", Zone::Battlefield);
    let spell = funded(&mut g, "Brass's Bounty");
    cast(&mut g, spell, None);
    passes(&mut g);
    let ts = tokens(&g, Player::P0, "Treasure");
    assert_eq!(ts.len(), 4);
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::ActivateMana{card,..}if *card==ts[0]))
        .unwrap();
    replay(&mut g, a);
    assert!(g.state.players[0].mana.iter().any(|&n| n > 0));
    assert_eq!(tokens(&g, Player::P0, "Treasure").len(), 3);
}

#[test]
fn germinate_uses_total_positive_gains_not_events_or_current_life_and_resets_each_turn() {
    let mut g = scenario();
    g.gain_life(Player::P0, 3);
    g.gain_life(Player::P0, 5);
    g.state.players[0].life -= 4;
    g.gain_life(Player::P0, 0);
    g.gain_life(Player::P0, -1);
    g.gain_life(Player::P1, 9);
    let p = &g.observe(Player::P0).players[0];
    assert_eq!(p.life, 24);
    assert_eq!(p.life_gain_events_this_turn, 2);
    assert_eq!(p.life_gained_this_turn, 8);
    let spell = funded(&mut g, "Germinate Recruits");
    cast(&mut g, spell, None);
    passes(&mut g);
    let cadets = tokens(&g, Player::P0, "Cadet"); // Cadet is a name, not a subtype.
    assert!(cadets.is_empty());
    let cadets: Vec<_> = g
        .state
        .objects
        .iter()
        .filter(|o| {
            o.zone == Zone::Battlefield && o.card.token_info().is_some_and(|t| t.name == "Cadet")
        })
        .collect();
    assert_eq!(cadets.len(), 8);
    assert!(
        cadets
            .iter()
            .all(|o| o.card.has_subtype("Wizard") && o.card.has_subtype("Soldier"))
    );
    g.begin_turn();
    assert_eq!(g.observe(Player::P0).players[0].life_gained_this_turn, 0);
    assert_eq!(g.observe(Player::P1).players[1].life_gained_this_turn, 0);
}

#[test]
fn life_gain_total_and_token_identity_overflows_restore_full_pending_state() {
    let mut g = scenario();
    let d = named(&mut g, Player::P0, "Professor Dellian Fel", Zone::Hand);
    g.move_object(d, Zone::Battlefield);
    g.state.players[0].life_gained_this_turn = u128::MAX;
    activate(&mut g, d, 0, None);
    replay(&mut g, Action::Pass);
    let before = g.snapshot();
    let rev = g.revision();
    assert!(matches!(
        g.apply(g.actor().unwrap(), Action::Pass),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
    assert_eq!(g.object_view(d).counters.loyalty, 7);
    for amount in [1u128 << 40, u128::MAX] {
        let mut g = scenario();
        g.state.players[0].life_gained_this_turn = amount;
        let spell = funded(&mut g, "Germinate Recruits");
        cast(&mut g, spell, None);
        replay(&mut g, Action::Pass);
        let before = g.snapshot();
        let rev = g.revision();
        assert!(matches!(
            g.apply(g.actor().unwrap(), Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), rev);
        let view = serde_json::to_value(g.observe(Player::P0)).unwrap();
        assert_eq!(
            view["players"][0]["life_gained_this_turn"].to_string(),
            amount.to_string()
        );
    }
}

#[test]
fn moogles_freezes_creature_count_before_creation_then_protects_both_old_and_new() {
    let mut g = scenario();
    let a = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let b = named(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let spell = funded(&mut g, "Moogles' Valor");
    cast(&mut g, spell, None);
    passes(&mut g);
    let ms = tokens(&g, Player::P0, "Moogle");
    assert_eq!(ms.len(), 2);
    for id in [a, b].into_iter().chain(ms) {
        assert!(g.keywords(id).contains(Keywords::INDESTRUCTIBLE));
    }
    for id in tokens(&g, Player::P0, "Moogle") {
        let o = g.object_view(id);
        assert_eq!((o.power, o.toughness), (1, 2));
        assert!(o.keywords.contains(Keywords::LIFELINK));
    }
}

#[test]
fn morcant_kindred_source_counts_itself_after_sacrifice_and_elf_cards_in_graveyard() {
    let mut g = scenario();
    let m = named(&mut g, Player::P0, "Morcant's Eyes", Zone::Battlefield);
    assert_eq!(g.object_view(m).card.kind_mask(), 128 | 32);
    assert!(!g.object_view(m).card.is_creature());
    named(&mut g, Player::P0, "Llanowar Elves", Zone::Graveyard);
    named(&mut g, Player::P0, "Three Tree Mascot", Zone::Graveyard);
    named(&mut g, Player::P1, "Llanowar Elves", Zone::Graveyard);
    g.state.players[0].mana = [0, 0, 0, 0, 2, 4];
    activate(&mut g, m, 0, None);
    assert_eq!(g.object_view(m).zone, Zone::Graveyard);
    passes(&mut g);
    let elves = tokens(&g, Player::P0, "Elf");
    assert_eq!(elves.len(), 3);
    for id in elves {
        let v = g.object_view(id);
        assert_eq!((v.power, v.toughness), (2, 2));
        assert_eq!(v.card.colors(), 20);
    }
    let mut g = scenario();
    named(&mut g, Player::P0, "Morcant's Eyes", Zone::Battlefield);
    g.begin_turn();
    g.prepare_triggers();
    passes(&mut g);
    let card = g.observe(Player::P0).library_choice.unwrap().remaining[0].id;
    replay(
        &mut g,
        Action::ArrangeLibrary {
            card,
            destination: LibraryDestination::Graveyard,
        },
    );
    assert_eq!(g.state.players[0].library.len(), 19);
}

#[test]
fn aatchik_counts_union_once_then_insect_deaths_grow_source_and_drain_opponent() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Three Tree Mascot", Zone::Graveyard);
    named(&mut g, Player::P0, "Bear Cub", Zone::Graveyard);
    named(&mut g, Player::P0, "Clachan Festival", Zone::Graveyard);
    named(&mut g, Player::P1, "Bear Cub", Zone::Graveyard);
    let a = funded(&mut g, "Aatchik, Emerald Radian");
    cast(&mut g, a, None);
    passes(&mut g);
    settle(&mut g);
    let insects = tokens(&g, Player::P0, "Insect");
    assert_eq!(insects.len(), 2);
    g.move_object(insects[0], Zone::Graveyard);
    settle(&mut g);
    assert_eq!(g.object_view(a).counters.plus_one, 1);
    assert_eq!(g.state.players[1].life, 19);
}

#[test]
fn sally_excludes_tokens_but_counts_itself_then_attack_counters_reach_every_owned_creature() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    g.create_tokens(Player::P0, factory("Goblin"), 2, false);
    let s = funded(&mut g, "Sally Pride, Lioness Leader");
    cast(&mut g, s, None);
    passes(&mut g);
    settle(&mut g);
    assert_eq!(tokens(&g, Player::P0, "Mutant").len(), 2);
    g.begin_turn();
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    replay(&mut g, Action::Attack { card: s });
    replay(&mut g, Action::FinishAttackers);
    settle(&mut g);
    for o in g
        .state
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Battlefield && o.card.is_creature())
    {
        assert_eq!(
            o.counters.plus_one,
            if o.owner == Player::P0 { 1 } else { 0 }
        );
    }
}

#[test]
fn squirrel_labels_and_all_three_effects_compile_to_actual_whole_gameplay() {
    let mut g = scenario();
    let s = funded(&mut g, "The Unbeatable Squirrel Girl");
    cast(&mut g, s, None);
    passes(&mut g);
    settle(&mut g);
    assert_eq!(tokens(&g, Player::P0, "Squirrel").len(), 1);
    g.state.players[0].mana = [0, 0, 0, 0, 3, 1];
    activate(&mut g, s, 0, None);
    passes(&mut g);
    assert_eq!(tokens(&g, Player::P0, "Squirrel").len(), 3);
    g.begin_turn();
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    replay(&mut g, Action::Attack { card: s });
    replay(&mut g, Action::FinishAttackers);
    settle(&mut g);
    assert_eq!(tokens(&g, Player::P0, "Squirrel").len(), 4);
}

#[test]
fn clachan_kindred_enchantment_keeps_both_fixed_bodies_and_creature_subtype() {
    let mut g = scenario();
    let c = funded(&mut g, "Clachan Festival");
    cast(&mut g, c, None);
    passes(&mut g);
    settle(&mut g);
    assert_eq!(tokens(&g, Player::P0, "Kithkin").len(), 2);
    assert_eq!(g.object_view(c).card.kind_mask(), 128 | 32);
    assert!(g.object_view(c).card.has_subtype("Kithkin"));
    assert!(!g.object_view(c).card.is_creature());
    g.state.players[0].mana = [1, 0, 0, 0, 0, 4];
    activate(&mut g, c, 0, None);
    passes(&mut g);
    assert_eq!(tokens(&g, Player::P0, "Kithkin").len(), 3);
}

#[test]
fn germinate_reads_later_gains_on_resolution_and_lifelink_counts_exact_event_amounts() {
    let mut g = scenario();
    g.gain_life(Player::P0, 3);
    let b = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let spell = funded(&mut g, "Germinate Recruits");
    cast(&mut g, spell, None);
    let moment = funded(&mut g, "Moment of Craving");
    cast(&mut g, moment, Some(Target::Card(b)));
    passes(&mut g);
    assert_eq!(g.state.players[0].life_gained_this_turn, 5);
    passes(&mut g);
    assert_eq!(tokens(&g, Player::P0, "Wizard").len(), 5);
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Bishop's Soldier", Zone::Battlefield);
    let b = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let source = g.damage_source(source);
    g.apply_damage_batch(&[
        DamageEvent {
            preventable: true,
            source,
            target: Target::Player(Player::P1),
            amount: 2,
            combat: true,
        },
        DamageEvent {
            preventable: true,
            source,
            target: Target::Card(b),
            amount: 3,
            combat: true,
        },
    ]);
    assert_eq!(g.state.players[0].life_gain_events_this_turn, 1);
    assert_eq!(g.state.players[0].life_gained_this_turn, 5);
    assert_eq!(g.state.players[0].life, 25);
}
