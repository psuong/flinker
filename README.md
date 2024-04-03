# FLinker

FLinker or File Linker is a small executable that lets you

* define a .yml file 
    * that points from the original file to a target file
    * specify the kind of file linkage
        * symlink
        * hard link

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
  - src: c.txt
```

## Build

```bash
cargo build --release
```
* Clone the repo and build it.
* Add flinker to your path

## Usage
```
flinker.exe example.yml
```
