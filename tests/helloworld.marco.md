---
name: Test Echo
author: Marco Polo
runner: {
    windows: Write-Output "Noice World",
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
