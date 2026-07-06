# known_failures

One file per external conformance suite (`anitomy_develop`, `anitomy_master`,
`anitopy`). Each lists, one per line, the fixture inputs that `anitomy_ng::parse`
does not currently match for that suite.

These are not bugs to fix blindly. The external suites have imperfect,
mutually-contradictory ground truth — some cases no implementation passes,
including upstream's own compiled binary (see `scripts/build_fixtures.py` and
`third_party/README.md`). The manifest turns the suite into a regression guard
plus a ratchet: both `anitomy/tests/conformance.rs` and
`anitomy-py/tests/test_conformance.py` fail if the failing set *changes* —
a newly-failing case is a regression; a newly-passing case must be removed here.

`self_rolled` has no file here: it is this project's own suite and must pass 100%.

Regenerate after a deliberate parser or fixture change (Rust is the source of
truth; the Python suite reads the same files):

```sh
UPDATE_KNOWN_FAILURES=1 cargo test -p anitomy-ng --test conformance
```

Then review the `git diff` — it shows exactly which cases moved and in which
direction. See `scripts/refresh_conformance.py` for the full re-vendor loop.
