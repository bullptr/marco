# marco

![GitHub release (latest by date)](https://img.shields.io/github/v/release/bullptr/marco)
![GitHub](https://img.shields.io/github/license/bullptr/marco)
![GitHub all releases](https://img.shields.io/github/downloads/bullptr/marco/total)
![GitHub repo size](https://img.shields.io/github/repo-size/bullptr/marco)
![GitHub stars](https://img.shields.io/github/stars/bullptr/marco)

A Markdown-based testing framework for command-line applications.

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

See this [example test file](https://github.com/bullptr/marco/blob/main/tests/python.marco.md) for more details on the test file format. Then run `marco` in the directory containing the test files to execute them.

## Install marco

Install prebuilt binaries via shell script

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/bullptr/marco/releases/latest/download/marco-installer.sh | sh
```

Install prebuilt binaries via powershell script

```sh
powershell -ExecutionPolicy Bypass -c "irm https://github.com/bullptr/marco/releases/latest/download/marco-installer.ps1 | iex"
```

## Download marco

| File                                                                                                                                       | Platform            | Checksum                                                                                                            |
| ------------------------------------------------------------------------------------------------------------------------------------------ | ------------------- | ------------------------------------------------------------------------------------------------------------------- |
| [marco-aarch64-apple-darwin.tar.xz](https://github.com/bullptr/marco/releases/latest/download/marco-aarch64-apple-darwin.tar.xz)           | Apple Silicon macOS | [checksum](https://github.com/bullptr/marco/releases/latest/download/marco-aarch64-apple-darwin.tar.xz.sha256)      |
| [marco-x86_64-apple-darwin.tar.xz](https://github.com/bullptr/marco/releases/latest/download/marco-x86_64-apple-darwin.tar.xz)             | Intel macOS         | [checksum](https://github.com/bullptr/marco/releases/latest/download/marco-x86_64-apple-darwin.tar.xz.sha256)       |
| [marco-x86_64-pc-windows-msvc.zip](https://github.com/bullptr/marco/releases/latest/download/marco-x86_64-pc-windows-msvc.zip)             | x64 Windows         | [checksum](https://github.com/bullptr/marco/releases/latest/download/marco-x86_64-pc-windows-msvc.zip.sha256)       |
| [marco-aarch64-unknown-linux-gnu.tar.xz](https://github.com/bullptr/marco/releases/latest/download/marco-aarch64-unknown-linux-gnu.tar.xz) | ARM64 Linux         | [checksum](https://github.com/bullptr/marco/releases/latest/download/marco-aarch64-unknown-linux-gnu.tar.xz.sha256) |
| [marco-x86_64-unknown-linux-gnu.tar.xz](https://github.com/bullptr/marco/releases/latest/download/marco-x86_64-unknown-linux-gnu.tar.xz)   | x64 Linux           | [checksum](https://github.com/bullptr/marco/releases/latest/download/marco-x86_64-unknown-linux-gnu.tar.xz.sha256)  |
