//! Actual entries, resolution ordinals and historical qualifying casts.
use super::*;
fn card(index: usize) -> Card {
    Card::catalogue()
        .nth(index)
        .expect("admitted token identity")
}
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Belladonna/Queen choice budget {:?}", g.state.pending);
}
fn finish(g: &mut Game) -> usize {
    let mut optional = 0;
    for _ in 0..64 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
        {
            return optional;
        }
        if g.legal_actions()
            .contains(&Action::ChooseOptional { accept: false })
        {
            optional += 1;
            act(g, Action::ChooseOptional { accept: false });
        } else {
            choose(g, |a| {
                matches!(
                    a,
                    Action::Pass | Action::PutTrigger { .. } | Action::FinishTargets
                )
            });
        }
    }
    panic!("Belladonna/Queen settle budget {:?}", g.state.pending);
}
fn actor(g: &mut Game, caster: Player, active: Player) {
    g.state.active = active;
    g.state.priority = caster;
    g.state.step = Step::Main1;
}
fn reset(g: &mut Game, p: Player) {
    g.state.active = p;
    g.begin_turn();
    g.state.pending = Pending::Priority;
    actor(g, p, p);
}
fn belladonna(g: &mut Game, p: Player) -> CardId {
    put(g, p, "Belladonna Took", Zone::Battlefield)
}
fn queen(g: &mut Game, p: Player) -> CardId {
    put(g, p, "The Queen of Dale", Zone::Battlefield)
}
fn tokens(g: &Game, p: Player, card: usize) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Battlefield && o.controller == p && o.card.index() == card)
        .count()
}
fn hop(g: &mut Game, p: Player) {
    actor(g, p, p);
    let spell = put(g, p, "Hop to It", Zone::Hand);
    g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 2];
    cast(g, spell, None);
}
fn one(g: &mut Game, p: Player) {
    g.create_tokens(p, card(5241), 1, false);
}
fn mox(g: &mut Game, p: Player) -> CardId {
    actor(g, p, p);
    let id = put(g, p, "Mox Jasper", Zone::Hand);
    cast(g, id, None);
    // Counter the actual cast to avoid an unrelated repeated legendary choice.
    counter_original(g, id);
    id
}
fn shock(g: &mut Game, caster: Player, active: Player) -> CardId {
    actor(g, caster, active);
    let id = put(g, caster, "Shock", Zone::Hand);
    g.state.players[caster.index()].mana = [0, 0, 0, 1, 0, 0];
    cast(g, id, Some(Target::Player(caster.other())));
    id
}
fn discard_recruit(g: &mut Game, p: Player, card: CardId) {
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectDiscard { .. })
    });
    assert_eq!(g.actor(), Some(p));
    act(g, Action::Discard { card });
}
fn counter_original(g: &mut Game, id: CardId) {
    let stack_id = g
        .state
        .stack
        .iter()
        .find(|i| i.source.is_some_and(|identity| identity.id == id) && i.kind == StackKind::Spell)
        .unwrap()
        .id;
    assert!(g.counter_stack_item(stack_id, false));
}
#[test]
fn belladonna_paid_body_actual_three_token_batch_and_existing_batch_consumer_stay_distinct() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = put(&mut g, p, "Belladonna Took", Zone::Hand);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
        cast(&mut g, b, None);
        finish(&mut g);
        assert_eq!(stats(&g, b), (2, 2));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        put(
            &mut g,
            p,
            "Mister Fantastic, Reed Richards",
            Zone::Battlefield,
        );
        one(&mut g, p.other());
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        hop(&mut g, p);
        assert_eq!(
            finish(&mut g),
            1,
            "old TokensEnterYou consumer still observes one batch"
        );
        assert_eq!(tokens(&g, p, 5285), 3);
        assert_eq!(g.state.players[p.index()].life, 21);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
        let rabbits: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.zone == Zone::Battlefield && o.card == card(5285))
            .map(|(i, _)| i as CardId)
            .collect();
        assert!(
            rabbits
                .iter()
                .all(|id| g.counter_count(*id, CounterKind::PlusOne) == 1)
        );
        one(&mut g, p);
        assert_eq!(finish(&mut g), 1);
        assert_eq!(g.state.players[p.index()].life, 21);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 1);
    }
}
#[test]
fn belladonna_countered_packet_does_not_consume_resolution_and_new_turn_resets() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = belladonna(&mut g, p);
        one(&mut g, p);
        until(&mut g, |g| {
            g.state.stack.iter().any(|s| {
                s.source.is_some_and(|source| source.id == b)
                    && matches!(s.kind, StackKind::TriggeredAbility { .. })
            })
        });
        let id = g
            .state
            .stack
            .iter()
            .find(|s| {
                s.source.is_some_and(|source| source.id == b)
                    && matches!(s.kind, StackKind::TriggeredAbility { .. })
            })
            .unwrap()
            .id;
        contracts::branch_contract(g.clone());
        assert!(g.counter_stack_item(id, false));
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].life, 20);
        hop(&mut g, p);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].life, 21);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 1);
        reset(&mut g, p.other());
        one(&mut g, p);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].life, 22);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
    }
}
#[test]
fn belladonna_queued_source_departure_and_new_incarnation_have_independent_ordinals() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = belladonna(&mut g, p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        for _ in 0..2 {
            one(&mut g, p);
            finish(&mut g);
        }
        one(&mut g, p);
        until(&mut g, |g| {
            g.state
                .stack
                .iter()
                .any(|s| s.source.is_some_and(|source| source.id == b))
        });
        let old = g.object_ref(b);
        g.move_object(b, Zone::Graveyard);
        contracts::branch_contract(g.clone());
        finish(&mut g);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 1);
        g.move_object(b, Zone::Battlefield);
        assert_ne!(g.object_ref(b), old);
        finish(&mut g);
        one(&mut g, p);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].life, 22);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 0);
    }
}
#[test]
fn belladonna_token_self_entry_and_bifur_added_instances_use_resolution_order() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.stage_copy_creation(
            p,
            CopyRecipe::new(Card::from_name("Belladonna Took").unwrap()),
            1,
        );
        finish(&mut g);
        assert_eq!(
            g.state.players[p.index()].life,
            21,
            "token Belladonna observes itself"
        );
        let mut g = scenario(p);
        let b = belladonna(&mut g, p);
        put(&mut g, p, "Bifur, Melodic Rider", Zone::Battlefield);
        put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        g.refresh_enduring_stories();
        let recipient = g.object_ref(b);
        let timestamp = g.next_role_timestamp();
        g.state
            .roles
            .get_or_insert_with(Default::default)
            .creature_types
            .push(subtypes::CreatureTypesEffectView {
                recipient,
                source: recipient,
                types: &["Dwarf"],
                additional: false,
                timestamp,
            });
        assert_eq!(g.object_view(b).additional_triggers, 1);
        one(&mut g, p);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].life, 21);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        one(&mut g, p);
        finish(&mut g);
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 1);
        assert_eq!(g.state.players[p.index()].life, 21);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
    }
}
#[test]
fn queen_paid_body_first_opponent_noncreature_after_creature_and_private_recruit_land_nonland() {
    for p in [Player::P0, Player::P1] {
        for land in [false, true] {
            let mut g = scenario(p);
            let q = put(&mut g, p, "The Queen of Dale", Zone::Hand);
            g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
            cast(&mut g, q, None);
            finish(&mut g);
            assert_eq!(stats(&g, q), (2, 1));
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            mox(&mut g, p);
            finish(&mut g);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            actor(&mut g, p.other(), p.other());
            let bear = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
            g.state.players[p.other().index()].mana = [0, 0, 0, 0, 1, 1];
            cast(&mut g, bear, None);
            finish(&mut g);
            let discard = put(&mut g, p, if land { "Forest" } else { "Shock" }, Zone::Hand);
            let drawn = put(&mut g, p, "Island", Zone::Library);
            let enemy = shock(&mut g, p.other(), p);
            counter_original(&mut g, enemy);
            g.move_object(q, Zone::Graveyard);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectDiscard { .. })
            });
            assert_eq!(g.actor(), Some(p));
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
            assert_eq!(g.object_view(drawn).zone, Zone::Hand);
            assert!(
                g.observe(p.other())
                    .objects
                    .iter()
                    .all(|o| o.id != drawn && o.id != discard)
            );
            contracts::branch_contract(g.clone());
            reject(&mut g, Action::Discard { card: bear });
            act(&mut g, Action::Discard { card: discard });
            finish(&mut g);
            assert_eq!(tokens(&g, p, 5241), usize::from(!land));
            g.move_object(q, Zone::Battlefield);
            shock(&mut g, p.other(), p);
            finish(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                1,
                "second noncreature cannot recruit after Queen returns"
            );
        }
    }
}
#[test]
fn queen_historical_first_cast_before_provider_and_turn_reset_are_per_caster() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        shock(&mut g, p.other(), p);
        finish(&mut g);
        queen(&mut g, p);
        mox(&mut g, p.other());
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        reset(&mut g, p.other());
        let discard = put(&mut g, p, "Forest", Zone::Hand);
        mox(&mut g, p.other());
        discard_recruit(&mut g, p, discard);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert_eq!(tokens(&g, p, 5241), 0);
        assert_eq!(g.state.players[p.other().index()].spells_cast_this_turn, 1);
    }
}
#[test]
fn queen_free_noncreature_cast_and_actual_adventure_face_are_qualifying_casts() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        queen(&mut g, p);
        put(&mut g, p.other(), "Omniscience", Zone::Battlefield);
        let discard = put(&mut g, p, "Forest", Zone::Hand);
        actor(&mut g, p.other(), p.other());
        let free = put(&mut g, p.other(), "Divination", Zone::Hand);
        g.state.players[p.other().index()].mana = [0; 6];
        act(
            &mut g,
            Action::AnnounceFree {
                card: free,
                face: CastFace::Normal,
                flash_surcharge: false,
            },
        );
        pay(&mut g);
        counter_original(&mut g, free);
        discard_recruit(&mut g, p, discard);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        let mut g = scenario(p);
        queen(&mut g, p);
        let discard = put(&mut g, p, "Forest", Zone::Hand);
        actor(&mut g, p.other(), p.other());
        let goblin = put(
            &mut g,
            p.other(),
            "Great Ugly-Looking Goblin // Clap! Snap!",
            Zone::Hand,
        );
        g.state.players[p.other().index()].mana = [0, 0, 1, 0, 0, 1];
        act(&mut g, Action::AnnounceAdventure { card: goblin });
        pay(&mut g);
        assert_eq!(
            g.state.players[p.other().index()].creature_spells_cast_this_turn,
            0
        );
        discard_recruit(&mut g, p, discard);
        finish(&mut g);
        assert_eq!(g.object_view(goblin).zone, Zone::Exile);
        actor(&mut g, p.other(), p.other());
        g.state.players[p.other().index()].mana = [0, 0, 1, 0, 0, 5];
        act(&mut g, Action::AnnounceExile { card: goblin });
        pay(&mut g);
        finish(&mut g);
        assert_eq!(
            g.state.players[p.other().index()].creature_spells_cast_this_turn,
            1
        );
        mox(&mut g, p.other());
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
    }
}
#[test]
fn queen_noncast_creature_copy_preserves_actual_history_and_recruit_occurs_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        queen(&mut g, p);
        actor(&mut g, p.other(), p.other());
        let bear = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
        cast(&mut g, bear, None);
        let discard = put(&mut g, p, "Forest", Zone::Hand);
        let sparks = put(&mut g, p.other(), "Choreographed Sparks", Zone::Hand);
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
        discard_recruit(&mut g, p, discard);
        finish(&mut g);
        assert_eq!(g.state.players[p.other().index()].spells_cast_this_turn, 2);
        assert_eq!(
            g.state.players[p.other().index()].creature_spells_cast_this_turn,
            1
        );
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        mox(&mut g, p.other());
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
    }
}
#[test]
fn queen_recruit_token_feeds_belladonna_but_land_discard_does_not() {
    let p = Player::P0;
    let mut g = scenario(p);
    queen(&mut g, p);
    belladonna(&mut g, p);
    let discard = put(&mut g, p, "Shock", Zone::Hand);
    mox(&mut g, p.other());
    discard_recruit(&mut g, p, discard);
    finish(&mut g);
    assert_eq!(tokens(&g, p, 5241), 1);
    assert_eq!(g.state.players[p.index()].life, 21);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
    reset(&mut g, p.other());
    let discard = put(&mut g, p, "Forest", Zone::Hand);
    mox(&mut g, p.other());
    discard_recruit(&mut g, p, discard);
    finish(&mut g);
    assert_eq!(tokens(&g, p, 5241), 1);
    assert_eq!(g.state.players[p.index()].life, 21);
}

#[test]
fn belladonna_noncreature_tokens_qualify_while_actual_nontoken_creatures_do_not() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = belladonna(&mut g, p);
        g.create_tokens(p, card(5302), 1, false);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].life, 21);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        let bear = put(&mut g, p, "Bear Cub", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
        cast(&mut g, bear, None);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        g.create_tokens(p, card(5302), 2, false);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 1);
        let treasures: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.zone == Zone::Battlefield && o.card == card(5302))
            .map(|(i, _)| i as CardId)
            .collect();
        assert!(
            treasures
                .iter()
                .all(|id| g.counter_count(*id, CounterKind::PlusOne) == 0)
        );
    }
}
