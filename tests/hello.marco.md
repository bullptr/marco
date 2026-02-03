---
name: Test Echo
author: Marco Polo
runner: { # @TODO support without curly braces
    windows: Write-Output,
    unix: echo,
    default: echo,
}
passing: true
date: 2026-01-28
---

# Test: Echo

## Input

```
Hello, world!
```

## Expected Output

```
Hello, world!
```
