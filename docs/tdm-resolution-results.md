# Resolution results and sampled milling

This engine checkpoint adds no card definitions or admission changes. It follows
the eight audited card shapes recorded in the paired evidence file.

`MillAmount::Amount` samples an ordinary resolving quantity once before a batch.
`MayMillAmount` guards its next instruction using the complete library count, so
an optional mill cannot be partly accepted. Mandatory milling does as much as
possible. Public destination redirection remains a completed mill; replacing a
move with a shuffle in the same library supplies no completed card departure.
The Yathan exact-four reflexive guard follows CR603.12 and the official release
note; its shuffle distinction is a reasoned CR614.6/701.17 inference.

`RecoverMilledOrSkip` is a choice-qualified continuation: an accepted exact
recovery skips the following fallback block, independently of replacement
results (CR118.12). It retains the existing recovery action and public eligible
post-mill incarnation checks.

`DamageByTargetPower` samples the legal power target at resolution start, before
another instruction moves it. The selected damage target group remains
independent. `ReturnToHandDrawForTokens` similarly captures token status before
the movement; optional/stale targets do not supply a token fact.

`DrawRemembered` uses the existing packet-owned, replacement-aware draw request.
`IfNoRememberedDraws` consumes that actual result; a guard that skipped the draw
also means no draw. The source counter fallback still requires its exact live
incarnation. New scalar consumers opt into all three existing quantity capture
walkers.

`DamageRememberPlayers` initializes one private result. Its batch owns the exact
resolving packet id across replacement choices; commit records only actual
positive player recipients. `DiscardDamagedPlayers` consumes the result through
the ordinary private discard continuation. Other damage batches cannot leak
into it. The separate life-gain instruction proceeds even if damage is prevented.

The new disjoint integration target includes actual paid casts/activation,
replacement and private-choice interactions, illegal-action rollback, exact
source/target incarnation boundaries, and owned raw/index/restore outcomes.
