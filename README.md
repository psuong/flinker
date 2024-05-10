# FLinker

FLinker or File Linker is a small executable that lets you

* define a .yml file 
    * that points from the original file / directory to a target file / directory
    * specify the kind of linkage between files / directories
        * symlink
        * hardlink
        * symlink-dir

## Example

Below is example of the yaml file.

```yml
# You can define either a hardlink or symlink between 2 files
hardlink:
  # The following can be a relative or absolute path
  - src: a.txt 
  - dst: b.txt
---
symlink:
  - src: a.txt
  - dst: c.txt
symlink-dir:
  - src: $HOME/a
  - dst: $HOME/b
```

## Build

```bash
cargo build --release
```
* Clone the repo and build it.
* Add flinker to your path

## Usage
```bash
flinker.exe example.yml
```
