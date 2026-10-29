use super::*;

fn face_create(g: &mut Game, p: Player, target: Player) {
    let spell = put(g, p, "Face Yourself", Zone::Hand);
    cast(g, spell, Some(Target::Player(target)));
    settle(g);
}
fn face_finish_end_step(g: &mut Game) {
    for _ in 0..128 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!(
        "Face Yourself triggers did not resolve: {:?}",
        g.state.pending
    );
}
fn face_tokens(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield && o.controller == p && o.is_copy_token())
                .then_some(id as CardId)
        })
        .collect()
}

#[test]
fn face_yourself_doubled_tokens_keep_their_copiable_sacrifice_abilities() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        face_create(&mut g, p, p.other());
        let copies = face_tokens(&g, p);
        assert_eq!(copies.len(), 2);
        for &token in &copies {
            assert!(g.keywords(token).contains(Keywords::HASTE));
            assert_eq!(
                g.copiable_recipe(g.object_ref(token))
                    .exceptions
                    .conditional_sacrifice_abilities,
                1
            );
        }
        g.begin_step(Step::End);
        assert_eq!(
            g.state
                .pending_triggers
                .iter()
                .filter(|s| s.kind == StackKind::CopyEndStepConditionalSacrifice)
                .count(),
            2
        );
        let saved = g.snapshot();
        for _ in 0..2 {
            face_finish_end_step(&mut g);
            assert!(copies.iter().all(|&token| !g.on_battlefield(token)));
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn face_yourself_copying_a_copy_retains_both_sacrifice_abilities() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        face_create(&mut g, p, p.other());
        let original = face_tokens(&g, p)[0];
        let source = g.object_ref(original);
        g.change_controller(source, p.other());
        face_create(&mut g, p, p.other());
        let copies = face_tokens(&g, p);
        let inherited = *copies
            .iter()
            .find(|&&id| {
                g.copiable_recipe(g.object_ref(id))
                    .exceptions
                    .conditional_sacrifice_abilities
                    == 2
            })
            .unwrap();
        assert!(g.keywords(inherited).contains(Keywords::HASTE));
        g.begin_step(Step::End);
        assert_eq!(
            g.state
                .pending_triggers
                .iter()
                .filter(|s| s.object == g.object_ref(inherited)
                    && s.kind == StackKind::CopyEndStepConditionalSacrifice)
                .count(),
            2
        );
        face_finish_end_step(&mut g);
        assert!(!g.on_battlefield(inherited));
    }
}

#[test]
fn face_yourself_condition_uses_controller_at_trigger_not_token_creator() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        face_create(&mut g, p, p.other());
        let token = face_tokens(&g, p)[0];
        let walker = put(
            &mut g,
            p.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        g.state.objects[walker as usize].counters.loyalty = 3;
        g.change_controller(g.object_ref(token), p.other());
        g.begin_step(Step::End);
        assert!(
            !g.state
                .pending_triggers
                .iter()
                .any(|s| s.object == g.object_ref(token)
                    && s.kind == StackKind::CopyEndStepConditionalSacrifice)
        );
        g.move_object(walker, Zone::Graveyard);
        settle(&mut g);
        assert!(g.on_battlefield(token));
    }
}

#[test]
fn face_yourself_losing_abilities_before_or_after_trigger() {
    for p in [Player::P0, Player::P1] {
        for before_step in [false, true] {
            let mut g = scenario(p);
            put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            face_create(&mut g, p, p.other());
            let token = face_tokens(&g, p)[0];
            if !before_step {
                g.begin_step(Step::End);
            }
            // Set up an existing aura effect at this boundary without passing
            // priority, so a previously queued trigger remains unresolved.
            let aura = put(&mut g, p, "Infinite Coursework", Zone::Battlefield);
            g.state.objects[aura as usize].attached_to = Some(g.object_ref(token));
            assert!(!g.printed_abilities_available(token));
            if before_step {
                g.begin_step(Step::End);
            }
            let count = g
                .state
                .pending_triggers
                .iter()
                .filter(|s| {
                    s.object == g.object_ref(token)
                        && s.kind == StackKind::CopyEndStepConditionalSacrifice
                })
                .count();
            assert_eq!(count, usize::from(!before_step));
            face_finish_end_step(&mut g);
            assert_eq!(g.on_battlefield(token), before_step);
        }
    }
}
