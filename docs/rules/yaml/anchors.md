# anchors

Check anchors and aliases.

## Options

- `forbid-undeclared-aliases`: (default: `true`) Forbid aliases that don't refer to an existing anchor.
- `forbid-duplicated-anchors`: (default: `true`) Forbid defining an anchor with a name already used by another anchor.
- `forbid-unused-anchors`: (default: `true`) Forbid anchors that are never used as an alias.

## Examples

### Correct

```yaml
anchor: &anchor value
alias: *anchor
```

### Incorrect (undeclared alias)

```yaml
alias: *undeclared
```

### Incorrect (duplicated anchor)

```yaml
anchor1: &anchor value
anchor2: &anchor value
```

### Incorrect (unused anchor)

```yaml
anchor: &unused value
```
