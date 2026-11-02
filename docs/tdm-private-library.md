# Private library continuations

`RefillEmptyLibraryFromGraveyard(n)` checks the controller's library after prior
instructions complete. It exiles only the exact live battlefield source, then
starts a mandatory non-targeted choice of n own graveyard cards, or every
available card if fewer exist. Source absence does not cancel that choice.

The existing owned `MilledSelection` supports a `LibraryTop` destination. Its
selected order is private: first selected is uppermost. Commit reorders only
successful new library incarnations and remembers the top privately using the
existing exact-top viewer mask. No card identities or ordering are revealed to
the opponent beyond the ordinary public graveyard departures (CR401.4).

`LookTopLandThenMayGraveyard` uses the existing private library selection frame.
A land offers a revealed hand move first. Declining advances the same exact card
to a separate optional graveyard phase. A nonland starts in that latter phase.
The `Keep` remainder leaves an unselected card in place; double decline retains
its private exact-top knowledge. Movement and shuffling use ordinary knowledge
invalidation. The new phase is owned inside the existing boxed frame.

These instructions use existing selection actions and pending variants. New
observation values describe the library-top destination, graveyard selection
and kept remainder; there are no new observation fields. Global schema and
inherited trace integration remain with the complete engine checkpoint.

The final matching-input target passes30 scenarios, including all22 earlier
resolution-result groups. Fourteen directly affected inherited private library,
copy selection, hideaway, random remainder and milled-continuation cases pass.
No card definitions, admissions or generated linkage changed.
