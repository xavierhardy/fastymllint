# comments

Enforce comment formatting.

## Options

- `require-starting-space`: (default: `true`) Require a space after the `#` of a comment.
- `ignore-shebangs`: (default: `true`) Ignore shebangs (like `#!/bin/bash`) at the beginning of files.
- `min-spaces-from-content`: (default: `2`) Minimum number of spaces between content and an inline comment.

## Examples

### Correct

```yaml
# This is a valid comment
key: value  # This is also valid
```

### Incorrect (missing starting space)

```yaml
#This comment is invalid
```

### Incorrect (too few spaces from content)

```yaml
key: value # This comment is invalid
```
