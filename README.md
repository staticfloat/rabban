# rabban

> "A carnivore never stops. Show no mercy. Never stop. Mercy is a chimera. It can be defeated by the stomach rumbling its hunger, by the throat crying its thirst. You must be always hungry and thirsty. Like me."

`rabban` is a resource monitor for squeezing every last drop of resources out of your CI machines, just as the Baron would have wanted.
It is intended to be started as a background task alongside a CI machine, and then sent a SIGINT signal to cleanup and exit at the end of the CI run.
It generates a `.csv` file that can be easily analyzed.

## Quickstart

Build `rabban` with `cargo build`, or download from one of our releases.
Run `rabban` with an output filename to begin collecting statistics:

```
./rabban test.csv
```

To adjust the time delay between statistics sampling, use `-t`.
For more options, use `--help`.

## Building for distribution

Use the `make multibuild` target to build binaries for all platforms, using the excellent [`cross` crate](https://github.com/cross-rs/cross).
