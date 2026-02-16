# sm-pkg

Experiment in trying to build a declarative "package manager" and configuration
tool for sourcemod.

See [DEVEL.md](DEVEL.md) for current, and very subject to change, implementation
notes & thoughts for future development.

## Plugins

The [plugins repository](https://github.com/sm-pkg/plugins) contains the source
to all of the available core plugins. Each plugin
contains a `plugin.yaml` file which describes the plugin and its dependencies.
The actual plugin code is located in the `src` directory.
These are kept separate from the `plugin.yaml` file to make it easier to manage
the source code and prevent any potential conflicts from
the upstream source trees.

## Commands

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
          --generate <GENERATOR>  [possible values: bash, fish, powershell, zsh]
      -h, --help                  Print help

The `sdk-*` commands are *not* used to install sourcemod inside a game folder,
its instead used for installation of the sdk used for compiling one-off plugins.
The install command should take care of installing the sourcemod (and metamod)
platforms into the game server path.

## sm-pkg-yaml

An example of a declarative game configuration.

```yaml
# yaml-language-server: $schema=https://raw.githubusercontent.com/sm-pkg/sm-pkg/refs/heads/master/schema/sm-pkg.json
---
game: TF
branch: Dev
plugins:
  - class_restrict
  - enablewhitelist
  - unusedvoicelines
create_startup_script: true
startup_opts:
  use_64bit: null
  mod_folder: tf
  port: null
  tv_port: null
  client_port: null
  max_players: null
  unrestricted_max_players: null
  map: null
  gslt: null
  rcon_password: null
  sv_password: null
  region: null
  ip: null
  sdr_enable: null
  workshop_authkey: asdf
templates:
  sourcemod_cfg:
    sm_show_activity: 16
    sm_menu_sounds: null
    sm_vote_delay: 10
    sm_datetime_format: null
    sm_immunity_mode: null
    sm_time_adjustment: null
    sm_flood_time: null
    sm_reserve_type: null
    sm_reserved_slots: null
    sm_hide_slots: null
    sm_chat_mode: null
    sm_timeleft_interval: null
    sm_trigger_show: null
    sm_vote_progress_hintbox: null
    sm_vote_progress_chat: null
    sm_vote_progress_console: true
    sm_vote_progress_client_console: false
  maplists_cfg:
    default_target: mapcyclefile
  databases_cfg:
    driver_default: sqlite
    databases:
    - name: storage-local
      database: sourcemod-local
      driver: sqlite
      host: null
      port: null
      user: null
      pass: null
      timeout: null
    - name: clientprefs
      database: clientprefs-sqlite
      driver: sqlite
      host: localhost
      port: null
      user: root
      pass: ''
      timeout: null
  core_cfg:
    logging: null
    log_mode: map
    log_time_format: null
    server_lang: null
    public_chat_trigger: null
    silent_chat_trigger: null
    silent_fail_suppress: null
    pass_info_var: null
    allow_cl_language_var: null
    disable_auto_update: null
    force_restart_after_update: null
    auto_update_url: null
    debug_spew: null
    steam_authstring_validation: null
    block_bad_plugins: null
    slow_script_timeout: null
    follow_csgo_server_guidelines: null
    jit_metadata: null
    enable_line_debugging: null
  admins_simple_ini:
    users:
    - identity: '76561197960287930'
      password: null
      flags: z
      immunity: 100
  admins_cfg:
    users:
    - name: the admin
      auth: steam
      identity: '76561197960287931'
      password: null
      group: null
      flags: null
      immunity: null
  admin_groups_cfg:
    default_immunity: null
    groups:
    - name: nerd group
      flags: a
      immunity: 50
      overrides:
      - command: sm_ban
        action: allow
  admin_overrides_cfg:
    overrides:
    - command: sm_csay
      flags: a
raw_configs:
- path: tf/cfg/server.cfg
  options:
    sv_maxcmdrate: '66'
    sv_minupdaterate: '15'
    net_splitpacket_maxrate: '100000'
    log: on
    sv_minrate: '80000'
    sv_mincmdrate: '15'
    sv_client_cmdrate_difference: '0'
    hostname: test server
    sv_maxupdaterate: '66'
    sv_maxrate: '0'
plugin_configs:
- path: tf/cfg/sourcemod/classrestrict.cfg
  options:
    sm_classrestrict_blu_demomen: '3'
```
