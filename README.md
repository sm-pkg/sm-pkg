# smpkg

Experiment in trying to build a "package manager" for sourcemod. Do not expect this to
become a "thing", it probably wont work like other attempts.


## Implementation Ideas; Nothing Concrete; All Subject To Change

Dont worry about complex dependency chains. The package depth is much more shallow compared to something like a OS.

Independent build roots. Each package has its own build root so its independent of other packages. This should allow
some minial level of "play" as far as versions of dependencies. My thoughts are going to evolve on this as i play with
this more. It would be nice to have it all unified like i keep my own plugin tree, but well see how feasible that can be
with a broader package set.

Enforce the use of `#pragma newdecls required` in all packages.

Track sourcemod stable and dev branches for support only. This means ensuring all plugins cleanly compile are *likely* to work, but no guarantee.

Some sort of package overlay system, maybe similar to nix, to allow for customizing packages.

Source is checked out from the repo and plugins are built locally as needed. This helps ensure that plugins are:

1. As reproducible as possible
2. Always able to be rebuilt over time, reducing bitrot with things like newer syntax.
3. Can be more easily audited.

There is currently no intention to support anything but source based distribution, however this may change in the future and support
for some sort of binary distribution could eventually be considered.

A per-project / srcds instance configuration and lockfile. Essentially this means allowing support for configuring multiple different instances
of a srcds. Maybe including some sort of presets, eg: all plugins required for mge?

Some sort of facility for generating configs, via a post-install script? Templates could be used to generate the
corresponding ini or kv files.

Enable/disable plugin for a installation.

Uninstallation of plugins? Not sure how possible this really is without expending too much effort on the feature. 

### Why Copy Sources And Not Use git repos or submodules?

While git and/or git submodules seems like a simple choice, but due to the nature of sourcemod the source code locations are all over the place. 
Many of these still only live as a post in the alliedmodders forum or somewhere similar. This makes discoverability a big challenge,
especially for newcomers. Having a central repository of at least the most popular plugins helps a lot with this.

Additionally, this helps actually preserve the plugins. Many become hard to find as links rot over time.

We also get the benefit of structuring the source tree in a standardized way for our purposes. Since the ecosystem is 
much smaller than say a language like C without a builtin package manager, we can afford to be more strict about the structure.

# Run

$ cargo run -- sourcemod install 1.13

# Previous work

- [SMAM](https://github.com/Phil25/SMAM)
- [SMAM2](https://github.com/Scags/SMAM2)
