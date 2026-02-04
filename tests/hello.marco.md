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

# Test: Echo 1

## Input

Foo bar.

```
Hello, world!
```

## Expected Output

```
Hello, world!
```

# Test: Echo 2

## Input

```
Hello, world!
```

## Expected Output

```
Hello, world!
```

# Test: Echo 3

## Input

```
Hello, world!
```

## Expected Output

```
Hello, world!
```

# Test: Echo 4

## Input

```
Hello, world!
```

## Expected Output

```
Hello, world!
```

# Test: Echo 5

## Input

```
Hello, world!
```

## Expected Output

```
Hello, world!
```
