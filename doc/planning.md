# Nirrpe Planning Sheet v0.1.1

## Schemas

Example Schema:
```toml
# schemas/character.toml

[schema]
version = 1
name = "Character"
# parent = ""

[key]
name = "Name"

[properties.flavor]
name = "Description"
type = "string"

[properties.hp]
name = "HP"
type = "double"
display = { type = "bar", foreground = ":red_heart:", background = ":black_heart:" }

[properties.mana]
name = "Mana"
type = "double"
display = { type = "bar", foreground = ":blue_square:", background = ":black_large_square:" }
```

Example object:
```toml
[character]
name = "Arcblroth"
flavor = "the Worldbuilder"
hp = 20
mana = 10

[bunny]
hop_strength = 9001
```

## Commands

### Syntax
`[a|b]` - pick 1 from
`{a}` - optional
`<a>` - replace with actual data
`<a...>` - optionally add more than one argument
`<name>` - object name

Every trait gets its own command prefix that is a shorthand for `/trait <name>`
```
/character Arcblroth set mana 9
```

---

```
/object <name> [add|remove] <trait name>
```
Adds or removes a trait from an object. **Removing traits may remove data**. All children of a trait are removed if the parent is removed.

```
/[trait|object] <name> [set] <attribute> <value>
```
Sets an attribute of an object. If used with `trait`, the attributes set are limited to that of the given trait and its children.

```
/[trait|object] <name> list <attribute> [add|insert|push|remove] <items...>
```
Adds or removes elements to a list attribute of an object. If used with `trait`, the attributes set are limited to that of the given trait and its children.

```
/[trait|object] <name> list <attribute> [give|take|steal|pilfer] <items...>
```
Adds or removes elements to a list attribute of an object. The corresponding elements are removed or added from the executor's list of the same attribute. If used with `trait`, the attributes set are limited to that of the given trait and its children.
\*`steal` and `pilfer` do the same thing as `take` and exist for RPG purposes.

```
/[trait|object] <name> [show|hide] <attribute>
```
Sets the visibility of this attribute. If used with `trait`, the attributes set are limited to that of the given trait and its children.

```
/[trait|object] <name> protect <attribute> [owner|party|global]
```
Sets the protection bits of this attribute. The people who can see and set data are restricted to
- `owner`:  only the owner of this object on it
- `party` only the people in the owner's current party
- `global` everyone

If used with `trait`, the attributes set are limited to that of the given trait and its children.

```
/[trait|object] <name> unprotect <attribute>
```
Sets the protection bits of this attribute to `global`.

```
/<trait name> create <name>
/object create <name> <traits...>
```
Starts the creation flow of an object (will ask for attributes in sequential order of the traits given).

```
/object [delete|undelete] <name>
```
Moves an object to or from the recycling bin. This will ask for confirmation.

```
/<trait name> <name>
/<trait name> <name> show
/object <name> show
```
Shows the information panel on an object. If used with a trait name, the information shown will be restricted th the attributes of the given trait and its children.

```
/party [add|remove|ban] <name>
```
\*ban does the same thing as `remove` and exists for RPG purposes.

```
/reload {script...}
```
Hot-reloads all scripts (if given no arguments), or the given script(s). Only executable by developers (see config file.)

## nirrpe_object

All objects implicitly include a trait called `nirrpe_object`, which is defined as
```toml
[properties.owner]
name = "Owner"
type = "snowflake"
display = { type = "none" }
```

## Scripts
this bot would also support custom scripts, so you would be able to do
```rs
// actions/battle.rs

use nirrpe::ObjectStore;
use serenity::model::interactions::application_command::ApplicationCommand;

async fn run(command: ApplicationCommand, objects: &mut ObjectStore) {}
```

scripts are hot-reloadable via `/reload`

## Hooks

```toml
[properties.flavor.hooks]
oncreate = "scripts/flavor_create.rs"
onupdate = "..."
ondelete = "..."
```

Schemas can specify "hooks", or scripts that are invoked any time an object with that trait changes. Each hook should have the signature
```rs
// scripts/flavor_create.rs

use nirrpe::ObjectStore;	
use nirrpe::runtime::Flavor;
use serenity::model::interactions::application_command::ApplicationCommand;

async fn run(command: ApplicationCommand, objects: &mut ObjectStore, name: &String, flavor: &mut Flavor) {}
```

## Bot Data File Structure

- `config/`
	- `actions/`
		- every file in this directory becomes a command
	- `schemas/`
		- every file in this directory because a trait
	- `scripts/`
		- every file in this directory becomes an invokable script
	- `resources/`
		- images that can be referred to in schemas or scripts
	- `alias.toml`
		- a map of commands to format strings to build other commands
	- `bot.toml`
		- bot token, owners, etc

- `data/`
		- all of the actual objects, in either toml or bson format (tbd)

- `cache/`
	- various compilation caches for scripts
