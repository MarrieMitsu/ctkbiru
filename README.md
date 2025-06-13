# ctkbiru

Simple command-line program to generate directory tree structure from a blueprint, part of my personal daily workflow utility.

`blueprint` simply means template in `txt` format.

## Installation

You need rust toolchain to build it yourself

```shell
cargo install --git https://github.com/MarrieMitsu/ctkbiru
```

## Usage

Define a tree structure blueprint file in `txt` format, you can also see the [blueprints](./blueprints) which contains list of my personal blueprint example.

`my-blueprint.txt` :

```text
assets/
 images/
 fonts/
  A/
   LICENSE.txt
  B/
   LICENSE.txt
src/
 this-is-a-dir/
 this-is-a-file
README.txt
```

Register the blueprint

```shell
ctkbiru add ./my-blueprint.txt
```

Be aware registering do not check whether the file contains any forbidden **printable ASCII characters**, **Non-printable characters**, **Reserved file names** and **Other rules** respective to your operating system. Instead it will produce an error when you try to generate the blueprint.

Generate the blueprint

```shell
ctkbiru gen my-blueprint --name my-project --path ./path/to/target
```

## License
[MIT](https://choosealicense.com/licenses/mit/)
