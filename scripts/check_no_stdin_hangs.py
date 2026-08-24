#!/usr/bin/env python3
"""No check may block on a read that never returns.

GC4 hung for two days at 0% CPU. Its ripgrep call carried a `--glob` but no path
argument, and ripgrep given no path reads *stdin*. Inside a command substitution
it inherited the script's stdin, so under a pipeline it blocked forever. A check
that never answers is worse than one that answers wrong: nothing downstream can
tell "still thinking" from "passed", and a runner without a timeout just waits.

This finds the shape rather than the instance — any command in `scripts/*.sh`
that would fall back to stdin because it was handed no file operand, including
inside `$(...)`, where the original bug lived.

The scanner calibrates itself against an embedded fixture before it reports.
A scanner that finds nothing because it cannot see is indistinguishable from a
clean tree, which is precisely the failure it exists to catch.
"""
import re, shlex, sys, pathlib, tempfile


READS_STDIN_WITHOUT_OPERAND = {
    "grep","egrep","fgrep","rg","sed","awk","gawk","sort","uniq","wc","cat",
    "head","tail","cut","jq","column","paste","join","comm","shasum","md5sum",
    "sha256sum","base64","tac","nl","fold","expand","rev",
}
ALWAYS_READS_STDIN = {"tr","xargs","tee"}

# Flags that consume the following token, per command. Arity is command
# specific and getting it wrong is how a scanner invents findings: `-n` takes a
# value for `head` and takes none for `grep`, so one table for all commands
# either swallows real operands or misses real bugs.
VALUED_BY_CMD = {
    "grep":  {"-e","-f","--regexp","--file","-m","--max-count","-A","-B","-C",
              "--after-context","--before-context","--context","--include",
              "--exclude","--color","--colour","-d","--devices","--directories"},
    "rg":    {"-e","-f","--regexp","--file","-m","--max-count","-A","-B","-C",
              "--after-context","--before-context","--context","-g","--glob",
              "-t","--type","-T","--type-not","--color","-M","--max-columns",
              "--iglob","-r","--replace"},
    "sed":   {"-e","--expression","-f","--file"},
    "awk":   {"-v","-f","-F","--assign","--file","--field-separator"},
    "jq":    {"--arg","--argjson","--slurpfile","--rawfile","--indent","-f",
              "--from-file","--tab"},
    "head":  {"-n","-c","--lines","--bytes"},
    "tail":  {"-n","-c","--lines","--bytes"},
    "cut":   {"-d","-f","-b","-c","--delimiter","--fields","--bytes","--characters"},
    "sort":  {"-k","-t","-o","-S","-T","--key","--field-separator","--output",
              "--buffer-size","--temporary-directory"},
    "uniq":  {"-f","-s","-w","--skip-fields","--skip-chars","--check-chars"},
    "comm":  {"--output-delimiter"},
    "join":  {"-1","-2","-j","-o","-t","-e"},
    "paste": {"-d","--delimiters"},
    "column":{"-c","-s","-t","--separator"},
    "nl":    {"-b","-f","-h","-i","-n","-s","-v","-w"},
    "fold":  {"-w","--width"},
}
DEFAULT_VALUED = {"-e","-f","--file","--regexp"}
VALUED = DEFAULT_VALUED  # kept for callers that ask for the union

# Commands whose first non-flag token is a program/pattern, not a file.
PROGRAM_ARG = {"awk":1,"gawk":1,"sed":1,"jq":1,"grep":1,"egrep":1,"fgrep":1,"rg":1}
# Once one of these supplied the program, no positional token is consumed for it.
PROGRAM_SUPPLIED_BY = {"-e","-f","--regexp","--file","--expression","--from-file"}


def segments(line):
    """Split into command segments, tracking whether a pipe feeds each.

    Splitting stops at `$(`...`)` boundaries: a substitution is its own command
    context, and — critically — it inherits the script's stdin.
    """
    out, buf, piped, depth, i, quote = [], "", False, 0, 0, None
    while i < len(line):
        c = line[i]
        if quote:
            buf += c
            if c == quote and line[i-1] != "\\":
                quote = None
            i += 1; continue
        if c in "'\"":
            quote = c; buf += c; i += 1; continue
        if line.startswith("$(", i):
            depth += 1; buf += "$("; i += 2; continue
        if c == ")" and depth:
            depth -= 1; buf += c; i += 1; continue
        if depth:                      # inside a substitution: do not split
            buf += c; i += 1; continue
        if c == "|" and not line.startswith("||", i):
            out.append((buf, piped)); buf = ""; piped = True; i += 1; continue
        if line.startswith("&&", i) or line.startswith("||", i):
            out.append((buf, piped)); buf = ""; piped = False; i += 2; continue
        if c == ";":
            out.append((buf, piped)); buf = ""; piped = False; i += 1; continue
        buf += c; i += 1
    out.append((buf, piped))
    return out


def unwrap(s):
    """Yield each command context in a segment: the segment itself, and the
    inside of any `$(...)` it contains — which is where GC4's bug lived.

    Quote-aware. A regex like `(fn|async_fn)` carries unbalanced-looking parens
    and a pipe; scanning for `)` without tracking quotes ends the substitution
    inside the pattern and hands `check` a truncated command.
    """
    yield s, False
    depth, start, i, quote = 0, None, 0, None
    while i < len(s):
        c = s[i]
        if quote:
            if c == quote and s[i-1] != "\\":
                quote = None
            i += 1; continue
        # Only single quotes suppress substitution. `"$(cmd)"` still runs cmd,
        # so treating double quotes as opaque hides every substitution in the
        # `x="$(...)"` form — which is the form GC4 used.
        if c == "'":
            quote = c; i += 1; continue
        if s.startswith("$(", i):
            if depth == 0:
                start = i + 2
            depth += 1; i += 2; continue
        if c == ")" and depth:
            depth -= 1
            if depth == 0 and start is not None:
                for sub, sub_piped in segments(s[start:i]):
                    yield sub, sub_piped
                start = None
        i += 1


def logical_lines(text):
    """Join continuation lines so a multi-line pipeline reads as one command.

    A line ending in `|`, `&&`, `||` or `\\` continues onto the next; scanning
    line by line makes every stage after the first look unpiped.
    """
    out, buf, first = [], "", None
    for n, raw in enumerate(text.splitlines(), 1):
        line = raw.rstrip()
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            if buf:
                out.append((first, buf)); buf, first = "", None
            continue
        if first is None:
            first = n
        buf = (buf + " " + stripped).strip() if buf else stripped
        if stripped.endswith(("|", "&&", "||", "\\")):
            continue
        out.append((first, buf)); buf, first = "", None
    if buf:
        out.append((first, buf))
    return out


def check(s, piped):
    s = s.strip()
    if not s or s.startswith("#"):
        return None
    s = re.sub(r'^[A-Za-z_][A-Za-z0-9_]*=', '', s).strip().strip('"')
    try:
        # Quote-aware: `awk '{print $1}'` is one token, not two. Splitting on
        # whitespace turns the program text into a phantom file operand.
        toks = shlex.split(s, posix=True)
    except ValueError:
        toks = s.split()
    if not toks:
        return None
    cmd = toks[0].lstrip("!").split("/")[-1]
    if cmd not in READS_STDIN_WITHOUT_OPERAND and cmd not in ALWAYS_READS_STDIN:
        return None
    if re.search(r'<\s*\S|<<<|<<', s):        # explicit stdin redirect
        return None
    if piped:
        return None
    if cmd in ALWAYS_READS_STDIN:
        return f"{cmd} always reads stdin and nothing pipes into it"

    valued = VALUED_BY_CMD.get(cmd, DEFAULT_VALUED)
    need_program = PROGRAM_ARG.get(cmd, 0)
    skip, operand = False, False
    for t in toks[1:]:
        if skip:
            skip = False; continue
        if t in valued:
            if t in PROGRAM_SUPPLIED_BY:
                need_program = 0
            skip = True; continue
        if t.startswith("-") and len(t) > 1:
            continue
        if need_program:
            need_program -= 1; continue
        operand = True
        break
    if not operand:
        return f"{cmd} has no file operand, so it reads stdin"
    return None


def scan(root):
    found = []
    # Recursive. This said `scripts/*.sh` and printed "no check can block on
    # stdin" while never looking at scripts/aws or scripts/azure — including the
    # campaign bootstrap that invokes Gate F. Claiming the tree while scanning
    # one directory of it is the same defect this scanner exists to find.
    for path in sorted(pathlib.Path(root).glob("scripts/**/*.sh")):
        for n, line in logical_lines(path.read_text()):
            seen = set()
            for seg, piped in segments(line):
                for ctx, ctx_piped in unwrap(seg):
                    why = check(ctx, piped or ctx_piped)
                    if why and why not in seen:
                        seen.add(why)
                        found.append((str(path), n, why, line))
    return found



# Every line here must be flagged. Line 2 is the historical GC4 bug verbatim.
MUST_CATCH = r"""#!/usr/bin/env bash
hits="$(rg -n --glob 'binn-data/src/{encoder,decoder}.rs' -e '^\s*(pub\s+)?fn\s+train\b' || true)"
grep -n 'TODO'
out=$(sort)
awk '{print $1}'
tr 'a' 'b'
jq '.status'
n="$(wc -l)"
xargs echo hello
cut -d, -f1
"""

# None of these may be flagged: fed by a pipe, given a file, or redirected.
MUST_IGNORE = r"""#!/usr/bin/env bash
hits="$(rg -n -e 'fn train' binn-data/src/encoder.rs </dev/null || true)"
grep -n 'TODO' src/main.rs
printf '%s\n' "$tree" | sort
cat file.txt | awk '{print $1}'
echo "x" | tr 'a' 'b'
jq '.status' report.json
wc -l < input.txt
grep -c foo <<< "$blob"
find . -name '*.rs' | xargs echo
shasum -a 256 "$f" |
  awk '{print $1}'
"""


def calibrate():
    """Prove the scanner sees, before believing that it saw nothing."""
    with tempfile.TemporaryDirectory() as d:
        root = pathlib.Path(d)
        (root / "scripts").mkdir()
        (root / "scripts" / "must_catch.sh").write_text(MUST_CATCH)
        (root / "scripts" / "must_ignore.sh").write_text(MUST_IGNORE)
        hits = scan(root)
    caught = {n for path, n, _, _ in hits if "must_catch" in path}
    false_positives = [h for h in hits if "must_ignore" in h[0]]
    expected = len([l for l in MUST_CATCH.splitlines() if l.strip()]) - 1
    ok = len(caught) == expected and not false_positives
    print(f"calibration: {len(caught)}/{expected} known-bad caught, "
          f"{len(false_positives)} false positives -> {'OK' if ok else 'NOT CALIBRATED'}")
    if not ok:
        missed = sorted(set(range(2, expected + 2)) - caught)
        if missed:
            print(f"  missed fixture lines: {missed}")
        for path, n, why, line in false_positives:
            print(f"  false positive at line {n}: {why}\n      {line[:110]}")
    return ok


# Commands that read stdin by design because they are pipeline filters. Each
# entry names the enclosing function and why it is safe; a filter function is
# only safe while every call site pipes into it, so the reason records where to
# look when that changes.
KNOWN_FILTERS = {
    ("scripts/check_gc2.sh", "sed has no file operand, so it reads stdin"):
        "body of crate_names(), a filter; all four call sites pipe into it",
}


def main() -> int:
    if not calibrate():
        print("refusing to report: the scanner is not calibrated")
        return 2
    root = pathlib.Path(__file__).resolve().parent.parent
    scanned = len(list(root.glob("scripts/**/*.sh")))
    if scanned < 20:
        print(f"refusing to report: found only {scanned} shell scripts; "
              "the scan root is wrong and a clean result would mean nothing")
        return 2
    unexplained = []
    for path, n, why, line in scan(root):
        rel = str(pathlib.Path(path).relative_to(root))
        reason = KNOWN_FILTERS.get((rel, why))
        if reason:
            print(f"known filter {rel}:{n}: {reason}")
        else:
            unexplained.append((rel, n, why, line))
    for rel, n, why, line in unexplained:
        print(f"HANG RISK {rel}:{n}: {why}\n    {line[:120]}")
    if unexplained:
        print(f"{len(unexplained)} command(s) would block on stdin. Give each an "
              f"explicit path operand, or redirect its stdin from /dev/null.")
        return 1
    print(f"no check can block on stdin ({scanned} shell scripts read)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
