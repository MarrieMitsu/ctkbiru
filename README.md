# ctkbiru

Simple command-line program to generate directory tree structure from a blueprint

## Usage

Define a tree structure blueprint file

`example.txt` :

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
$ ctkbiru add ./example.txt
```

Generate the blueprint

```shell
$ ctkbiru gen example -p ./path/to/target
```

## Notes

I can not guarantee this program will run seamlessly, because I made this program only to meet my daily needs.

## License
[MIT](https://choosealicense.com/licenses/mit/)
