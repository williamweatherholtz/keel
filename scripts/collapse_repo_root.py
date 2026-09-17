"""sprint 734: collapse every local `fn repo_root()` onto keel_fs::test_support::repo_root and repoint
keel-cli/src's cwd-relative test anchors to it. Idempotent: a file already in the new shape is skipped.
not-an-instrument: it is a one-shot codemod (the migration skill's gate 1); the counts it prints are the
transform reporting what it rewrote, not a measure of the project - the anchor scan in keel-cli/src/touched.rs
(no_member_test_anchors_on_a_cwd_relative_path) and the D0388 pair pos734/neg734 are the sensors it answers to."""
import re
import sys

USE = "    use keel_fs::test_support::repo_root;\n"
HELPERS = [
    "members/keel-github/src/github.rs",
    "members/keel-model/src/activation.rs",
    "members/keel-model/src/fingerprint.rs",
    "members/keel-model/src/orient.rs",
    "members/keel-model/src/queries.rs",
    "members/keel-schema/src/cli_surface.rs",
    "members/keel-view/src/arch.rs",
    "members/keel-view/src/view/mod.rs",
    "members/keel-write/src/write.rs",
]
DOC_HEADS = ("    /// The repository root, found from the crate manifest", "    /// The checkout root: the first ancestor")
removed = 0
for p in HELPERS:
    lines = open(p, encoding="utf-8").read().split("\n")
    while True:
        idx = next((i for i, l in enumerate(lines) if l == "    fn repo_root() -> std::path::PathBuf {"), None)
        if idx is None:
            break
        end = idx
        while lines[end] != "    }":
            end += 1
        start = idx
        # the helper's own doc: walk up through `///` lines until the line that opens it (inclusive)
        j = idx - 1
        while j >= 0 and lines[j].startswith("    ///"):
            if lines[j].startswith(DOC_HEADS):
                start = j
                break
            j -= 1
        else:
            sys.exit(f"{p}: no doc head above fn repo_root at line {idx + 1}")
        # one blank line after the body goes with it (the next item keeps its own spacing)
        if lines[end + 1] == "":
            end += 1
        # the enclosing test module: the nearest `mod <name> {` above, under a #[cfg(test)]
        m = next(i for i in range(idx, -1, -1) if re.fullmatch(r"mod \w+ \{", lines[i]) and lines[i - 1] == "#[cfg(test)]")
        del lines[start : end + 1]
        lines.insert(m + 1, USE.rstrip("\n"))
        removed += 1
        print(f"{p}: removed fn repo_root at {idx + 1} (doc from {start + 1}), use inserted after line {m + 1}")
    open(p, "w", encoding="utf-8", newline="\n").write("\n".join(lines))

# the two members with no keel-fs edge gain the dev-dependency keel-actor already carries (D0498's shape)
DEV = '\n[dev-dependencies]\n# keel_fs::test_support::repo_root, the one repository-root walk for tests (issue557); a leaf depending on a leaf, downward.\nkeel-fs = { path = "../keel-fs" }\n'
for p in ("members/keel-github/Cargo.toml", "members/keel-schema/Cargo.toml"):
    t = open(p, encoding="utf-8").read()
    if "keel-fs" in t:
        print(f"{p}: already depends on keel-fs")
        continue
    t = t.rstrip("\n") + "\n" + DEV
    open(p, "w", encoding="utf-8", newline="\n").write(t)
    print(f"{p}: dev-dependency added")

# keel-cli/src: the eight anchors, each a whole line
ANCHORS = {
    "keel-cli/src/adherence.rs": [('        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");', "        let root = keel_fs::test_support::repo_root();")],
    "keel-cli/src/attestation.rs": [('        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");', "        let root = keel_fs::test_support::repo_root();")],
    "keel-cli/src/main.rs": [('        let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");', "        let repo = keel_fs::test_support::repo_root();")],
    "keel-cli/src/process_cmd.rs": [('        let root = Path::new("..");', "        let root = &keel_fs::test_support::repo_root();")],
    "keel-cli/src/reports.rs": [
        ("        // D0087: each report yields a non-empty cards array; unknown report errors. (cwd = crate dir.)", "        // D0087: each report yields a non-empty cards array; unknown report errors."),
        ('        let root = std::path::Path::new("..");', "        let root = &keel_fs::test_support::repo_root();"),
    ],
    "keel-cli/src/serve.rs": [('        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");', "        let root = keel_fs::test_support::repo_root();")],
    "keel-cli/src/workspace.rs": [('        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");', "        let root = keel_fs::test_support::repo_root();")],
}
repointed = 0
for p, pairs in ANCHORS.items():
    t = open(p, encoding="utf-8").read()
    for old, new in pairs:
        n = t.count(old + "\n")
        if n == 0 and t.count(new + "\n"):
            print(f"{p}: already repointed")
            continue
        assert n >= 1, (p, old)
        t = t.replace(old + "\n", new + "\n")
        repointed += n
        print(f"{p}: {n} repointed")
    open(p, "w", encoding="utf-8", newline="\n").write(t)
print(f"helpers removed {removed}; anchors repointed {repointed}")
