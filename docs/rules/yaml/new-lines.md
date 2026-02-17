# new-lines

Enforce a consistent newline type throughout the file.

## Options

- `type`: (default: `unix`) The desired newline type. Can be `unix` (`
`), `dos` (`
`), or `platform` (system default).

## Examples

### Correct (`type: unix`)

```yaml
key: value
other: value
```

### Incorrect (`type: unix`)

```yaml
key: value

other: value

```
