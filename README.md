# sm-pkg

Experiment in trying to build a declarative "package manager" and configuration tool for sourcemod. 

See [DEVEL.md](DEVEL.md) for current, and very subject to change, implementation notes & thoughts for future development.

## Plugins

The [plugins repository](https://github.com/sm-pkg/plugins) contains the source to all of the available core plugins. Each plugin
contains a `plugin.yaml` file which describes the plugin and its dependencies. The actual plugin code is located in the `src` directory.
These are kept separate from the `plugin.yaml` file to make it easier to manage the source code and prevent any potential conflicts from
the upstream source trees.

## Commands

```
Usage: sm-pkg [OPTIONS] <COMMAND>

Commands:
  init         Initialize a new project
  install      Install all project dependencies
  add          Add one or more plugins to a project
  remove       Remove one or more plugins from a project
  config       Generate configuration files
  list         List configured project pacakges
  search       Search package cache
  build        Build one or more plugins
  update       Update package cache
  sdk-install  Download and install sourcemod
  sdk-list     List installed sourcemod versions
  sdk-latest   Fetches the latest version of sourcemod for a branch
  build-index  Rebuild the package index in the local directory
  help         Print this message or the help of the given subcommand(s)

Options:
  -a, --app-root <APP_ROOT>   [default: ~/.sm-pkg]
      --generate <GENERATOR>  [possible values: bash, elvish, fish, powershell, zsh]
  -h, --help                  Print help
```

The `sdk-*` commands are *not* used to install sourcemod inside a game folder, its instead used for installation of the sdk used for compiling one-off plugins. The
install command should take care of installing the sourcemod (and metamod) platforms into the game server path.
