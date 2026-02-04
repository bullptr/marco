---
name: Multi-line values test
runner: { windows: Write-Output, unix: echo, default: echo }
---

## Hello World x3

Input:

```
Hello, world!
Hello, world!
Hello, world!
```

Expected Output:

```
Hello, world!
Hello, world!
Hello, world!
```
