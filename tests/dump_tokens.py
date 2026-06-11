#!/usr/bin/env python3
"""Dump PyYAML's token stream in the same canonical format as the Rust
`dump_tokens` binary, for differential testing."""

import sys

import yaml

sys.path.insert(0, "tests")


def main():
    path = sys.argv[1]
    with open(path, "rb") as f:
        data = f.read()

    # Same auto-decode as yamllint
    from yamllint.decoder import auto_decode
    content = auto_decode(data)

    loader = yaml.BaseLoader(content)
    try:
        while True:
            token = loader.get_token()
            if token is None:
                break
            name = type(token).__name__.removesuffix("Token")
            value = ""
            if isinstance(token, yaml.DirectiveToken):
                value = f"{token.name} {format_directive(token.value)}"
            elif isinstance(token, (yaml.AliasToken, yaml.AnchorToken)):
                value = token.value
            elif isinstance(token, yaml.TagToken):
                handle, suffix = token.value
                value = f"({handle if handle is not None else 'None'}, {suffix})"
            elif isinstance(token, yaml.ScalarToken):
                style = token.style if token.style is not None else "None"
                value = (
                    f"plain={'true' if token.plain else 'false'} "
                    f"style={style} value={rust_debug(token.value)}"
                )
            s, e = token.start_mark, token.end_mark
            print(
                f"{name} [{s.pointer},{s.line},{s.column}]"
                f"-[{e.pointer},{e.line},{e.column}] {value}".rstrip()
            )
    except yaml.error.MarkedYAMLError as err:
        m = err.problem_mark
        if m is not None:
            print(f"ERROR {err.problem} [{m.pointer},{m.line},{m.column}]")
        else:
            print(f"ERROR {err.problem}")


def format_directive(value):
    if value is None:
        return "None"
    a, b = value
    return f"({a}, {b})"


def rust_debug(s):
    """Format a string like Rust's {:?}."""
    out = ['"']
    for ch in s:
        if ch == '"':
            out.append('\\"')
        elif ch == "\\":
            out.append("\\\\")
        elif ch == "\n":
            out.append("\\n")
        elif ch == "\r":
            out.append("\\r")
        elif ch == "\t":
            out.append("\\t")
        elif ch == "\0":
            out.append("\\0")
        elif ch.isprintable():
            out.append(ch)
        else:
            code = ord(ch)
            out.append(f"\\u{{{code:x}}}")
    out.append('"')
    return "".join(out)


if __name__ == "__main__":
    main()
