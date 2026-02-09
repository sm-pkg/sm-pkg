# Development and Implentation Notes

This basically outlines my current vision for the project.  Much of this is very likely to change as things
get ironed out more and more pros and cons become apparent.

## Dependencies

Dont worry about complex dependency chains, be lax about versioning. The package depth is much more shallow compared to something like a OS. I dont
think any plugins would really pull in more than 2 levels deep of a dependency? Sourcemod is very flexible as far as plugins and versioning, i think
as long as the type signatures match thats effectively all that matters outside of major sm version differences? 


## Independent Build Roots

Each package has its own build root so its independent of other packages. This should allow
some minial level of "play" as far as versions of dependencies. My thoughts are going to evolve on this as i play with
this more. It would be nice to have it all unified like i keep my own plugin tree, but well see how feasible that can be
with a broader package set.


## Enforce newdecls

Enforce the use of `#pragma newdecls required` in all plugins. This has a number of benefits, including helping to ensure that all 
plugins are up-to-date with the latest syntax and language features. This forces any old plugins to be brought up to date
with contemporary syntax, which helps reduce bitrot.

## Sourcemod Versions

Track sourcemod stable and dev branches for support only. This means ensuring all plugins cleanly compile are *likely* to work, but no guarantee.

## Per Project/SRCDS Instance Configuration

A per-project / srcds instance configuration and lockfile. Essentially this means allowing support for configuring multiple different instances
of a srcds. Maybe including some sort of presets, eg: all plugins required for mge? This should follow a familiar workflow similar to tools 
like npm, apt, etc.

### Lockfile

Include a lockfile for plugin versioning, similar to any programming language.

### Config Generator

Generates sourcemod and game configuration files using a declarative yaml config format. 

## Source Based Plugin Repository

Source is checked out from the repo and plugins are built locally as needed. This helps ensure that plugins are:

1. As reproducible as possible
2. Always able to be rebuilt over time, reducing bitrot with things like newer syntax.
3. Can be more easily audited.

There is currently no intention to support anything but source based distribution, however this may change in the future and support
for some sort of binary distribution could eventually be considered. Ideally the tool will be easy enough to use that there should be no
need to use pre-compiled plugins.

### Custom Package Overlay

Some sort of package overlay system to allow for customizing packages or adding private plugins.

### Why Copy Sources And Not Use git repos or submodules?

While git and/or git submodules seems like a simple choice, but due to the nature of sourcemod the source code locations are all over the place. 
Many of these still only live as a post in the alliedmodders forum or somewhere similar. This makes discoverability a big challenge,
especially for newcomers. Having a central repository of at least the most popular plugins helps a lot with this.

Additionally, this helps actually preserve the plugins. Many become hard to find as links rot over time.

We also get the benefit of structuring the source tree in a standardized way for our purposes. Since the ecosystem is 
much smaller than say a language like C without a builtin package manager, we can afford to be more strict about the structure (i hope).

Ultimately it would be nice to use a model similar to golang which can point to arbitrary git repos, most of the ecosystem
has not however migrated to this model yet. That said, most actively developed and used stuff likely is. We will likely end up 
with some sort of hybrid model.
