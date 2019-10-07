MORP
====

`morp` is a command line that provides utilities for working with javascript monorepos.

Installation
------------

 1. Install [rustup](https://rustup.rs/) to have a working rust environment.
 2. Run `cargo install morp`

You may need to add the Cargo bins install directory to your `$PATH` to be able to run `morp`.

Prerequisites
-------------

 - The packages of your monorepo must be in a `packages` directory located at the root of the monorepo.
 - The `name` key in the `package.json` file of your packages must match the name of the package directory in `packages`
 
```
monorepo
├── package.json
├── packages
│   ├── foo
│   │   ├── package.json -> `name` key must be `foo`
│   │   └── ...
│   └── ...
└── ...
```

Subcommands
-----------

### morp graph

This command outputs a graph of the interdependencies between the monorepo packages.

#### options

`--path -p` path of the monorepo

### morp diff

Print the name of the changed packages compared to a branch, taking into account dependent packages.
Let's say you have 3 packages, `A`, `B` and `C`. Package `B` depends on `A` and package `C` doesn't depend on anything.
If you make a change in package `A`, then `morp diff` will output this:
```
A
B
```

This can be useful for CI purposes, if you want to trigger jobs depending on changed packages.

This command works by first looking for a common ancestor between `HEAD` and the branch you specified (or `develop` by default).
Once we have this common ancestor (`ca`), we check which packages changed between `ca` and `HEAD` and the packages depending on these.

If there are changes outside of the monorepo packages, the `root` keyword will be output.

#### options

`--path -p` path of the monorepo

`--branch -b` branch to use as a reference for changed packages

`--prefix` add a prefix to each package in the packages output


Caveat
------

If you have a directory in the `packages` directory that doesn't have a `package.json` file, `morp` won't be able to work correctly.
