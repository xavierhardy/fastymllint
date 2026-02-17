# quoted-strings

Enforce string quoting rules.

## Options

- `quote-type`: (default: `any`) Enforce a specific quote type (`single`, `double`, `consistent`, `any`).
- `required`: (default: `only-when-needed`) Control when quotes are required (`true`, `false`, `only-when-needed`).
- `extra-required`: (default: `[]`) List of strings that must always be quoted.
- `extra-allowed`: (default: `[]`) List of strings that are allowed to be unquoted even if `required` is `true`.
- `allow-quoted-quotes`: (default: `false`) Allow quotes within quoted strings.
- `check-keys`: (default: `false`) Also check keys for quoting rules.

## Examples

### Correct

```yaml
key: "value"
another_key: 'another value'
```

### Incorrect

```yaml
# With quote-type: single
key: "value"
```
