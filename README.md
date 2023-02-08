# Chewy
A WinRT wrapper for [Taffy](https://github.com/DioxusLabs/taffy).

This project is an experiment in exposing code written in Rust via the WinRT ABI. Chewy is meant to be primarily consumed by C# callers via a nuget package. To build a nuget package from this repo, use the [nuget_rust](https://github.com/robmikh/nuget_rust) tool:

```console
git clone https://github.com/robmikh/Chewy
cd Chewy
# This will build the project for x64 and ARM64, and then packages them up.
nuget_rust -a
```
