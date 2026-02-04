# marco

A Markdown-based test framework

## Usage

```
Usage: marco.exe [OPTIONS]

Options:
  -i, --input <INPUT>    Glob or direct file for test collection [default: **/*.marco.md]
  -r, --runner <RUNNER>  Command to run the tests with (overridden by "runner" field in test header)
      --threads <N>      Maximum number of threads to use in parallel (default: num_cpus) [env: MARCO_MAX_THREADS=]
  -v, --verbose          Verbose output
  -h, --help             Print help
```
