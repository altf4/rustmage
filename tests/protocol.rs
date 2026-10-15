use rustmage::protocol::{Request, Session};
use rustmage::{Card, Config, DeckRules, Player};

fn session() -> Session {
    Session::new(Config {
        decks: [vec![Card::Mountain; 60], vec![Card::Mountain; 60]],
        seed: 1,
        starting_player: Player::P0,
        deck_rules: DeckRules::Freeform,
    })
    .unwrap()
}

#[test]
fn protocol_rejects_invalid_requests_without_mutating_game() {
    let mut session = session();
    let before = session.json(r#"{"op":"observe","viewer":"P0"}"#);
    for request in [
        r#"{"op":"unknown"}"#,
        r#"{"op":"observe","viewer":"P2"}"#,
        r#"{"op":"step","actor":"P0","viewer":"P0","revision":0,"index":999999}"#,
        r#"{"op":"step","actor":"P1","viewer":"P1","revision":0,"index":0}"#,
        r#"{"op":"observe","viewer":"P0","unexpected":true}"#,
        "invalid json",
    ] {
        let result: serde_json::Value = serde_json::from_str(&session.json(request)).unwrap();
        assert!(result["error"].is_string());
        assert_eq!(session.json(r#"{"op":"observe","viewer":"P0"}"#), before);
    }
}

#[test]
fn snapshot_drop_and_stale_actions_are_explicit_errors() {
    let mut session = session();
    let snapshot = session.request(Request::Snapshot).snapshot.unwrap();
    assert!(
        session
            .request(Request::Restore { snapshot })
            .error
            .is_none()
    );
    let response = session.request(Request::Step {
        actor: Player::P0,
        revision: 0,
        index: 1,
        viewer: Player::P0,
    });
    assert!(response.error.unwrap().contains("StaleRevision"));
    assert!(
        session
            .request(Request::DropSnapshot { snapshot })
            .error
            .is_none()
    );
    assert!(
        session
            .request(Request::Restore { snapshot })
            .error
            .is_some()
    );
}
