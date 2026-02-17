# comments-indentation

Enforce that comments are indented at the same level as the content around them.

## Examples

### Correct

```yaml
key:
  # This comment is correctly indented
  value: 1
```

### Incorrect

```yaml
key:
# This comment is not correctly indented
  value: 1
```
