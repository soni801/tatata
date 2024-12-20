# TATATA

Total Automation for Tasks and Actions using Typing and Aiming

---

## What?

A cross-platform scripting language for executing keyboard and mouse events.

## Why?

Because there are no other easily available tools that does exactly this. Auto clickers are too simple, and AutoHotkey
is too advanced.

## How?

Download the latest [release](https://github.com/soni801/tatata/releases) for your platform. Open a command prompt in
the same directory as your _tatata_ executable is in. Create a `.tatata` file containing your script, and run
`tatata.exe script.tatata` to run the script (replacing `tatata.exe` with the correct executable for your platform).

### Additional instructions for macOS

> [!IMPORTANT]
> TATATA currently only runs on Macs running Apple Silicon SoC's.

Before running TATATA, you'll need to make it executable by running `chmod +x ./tatata-macos` (replacing `tatata-macos`
with the name of the executable). When running TATATA for the first time, your Mac will likely complain about Apple not
being able to check the executable for malware. To resolve this, go to _System Settings > Privacy & Security_ and
approve TATATA towards the bottom of the page.

> [!TIP]
> If you're afraid of manually approving software, you can read through the code to assure that it is safe to run.
> Alternatively, you can compile the app yourself from source.

## The TATATA language

The TATATA language is heavily inspired by the [Portal 2 TAS language](https://wiki.portal2.sr/TASing). Every line
consists of a timestamp and an action field, separated by an angle bracket (`>`).

The timestamp is the point in time to execute the associated actions. There are two types of timestamps:

- _Absolute timestamps_, only a number: The specific point in time to execute the actions, in milliseconds after the
  script execution was started. For example, `3000>` will execute 3000ms after the execution was started.
- _Relative timestamps_, a `+` followed by a number: A certain time after the previous action line, in milliseconds. For
  example, `+50>` will execute 50ms after the previous action line.

The action field can contain any number of actions separated by a semicolon (`;`). Valid actions are:

- `mousemove`: Move the mouse to the specified absolute position. Takes 3-4 arguments:
  - Movement method (`abs`/`rel`); whether the mouse should move to a specific position or relative to its current position
  - X coordinate/distance
  - Y coordinate/distance
  - Time _(optional)_: the time it should take for the cursor to move to the specified location, in milliseconds.
    Defaults to 0 (instantly snaps) if unset.
- `mousedown`/`mouseup`: Respectively press or release the specified mouse button:
  - `1`: Left click
  - `2`: Right click
  - `3`: Middle click
  - `4`: Back (unavailable on macOS)
  - `5`: Forward (unavailable on macOS)
- `keydown`/`keyup`: Respectively press or release a key on the keyboard. This can take any of the following:
  - A letter or number found on a standard keyboard
  - A symbol you can type on the base layer of your keyboard, i.e. without holding any modifiers
  - The following function keys: `f1`, `f2`, `f3`, `f4`, `f5`, `f6`, `f7`, `f8`, `f9`, `f10`, `f11`, `f12`, `f13`,
  `f14`, `f15`, `f16`, `f17`, `f18`, `f19`, `f20`
  - The following modifiers: `control`, `shift`, `alt`, `super` (windows/command key), `capslock`
  - Arrow keys: `up`, `down`, `left`, `right`
  - The following other special keys: `tab`, `escape`, `space`, `enter`, `backspace`, `insert` (unavailable on macOS),
  `delete`, `home`, `end`, `pageup`, `pagedown`
- `release`: Release all currently held inputs of the specified type:
  - `mouse`: Release all held mouse buttons
  - `key`: Release all held keyboard keys
  - `both`: Release all held mouse buttons and keyboard keys
- `text`: Write the following text, up until the end of the line or the next semicolon (`;`). Does not need to be
  wrapped in quotes, and cannot contain the angle bracket separator (`>`).

There's also two types of comments:

- Line comments, opened with `//`: The rest of the line will be considered a comment and not be parsed
- Block comments, opened with `/*` and closed with `*/`: The text between the opening and closing slashes will be
  considered a comment and not be parsed. This applies regardless of whether the opening and closing slashes are on the
  same line.

### Example

```
/* This script does something cool
And I can type stuff here because this is a comment */

// Do some stuff
0>mousemove abs 500 100; mousedown 1;mouseup 1
100>keydown enter;keyup enter;text Hello World! // This types Hello World amongst other things
200>mousemove rel 300 0;mousedown 2
+50>mousemove rel 0 570 200;mouseup 2
500>mousemove abs 1460 120

+100>keydown a; keydown b; keydown c
// Now, let's release all the keys that we are holding down
+500>release key
```

## Future plans

TATATA is obviously still in super early development. You can find future plans, as well as an overview of the current
status, on the [GitHub Project](https://github.com/users/soni801/projects/6).

> [!CAUTION]
> Due to TATATA still being early in its development cycle, expect many upcoming changes to break existing scripts.
> If this is a concern, you can keep older versions of TATATA on your computer for usage with older scripts.
> Alternatively, you can download older releases from GitHub as I intend on keeping the version history intact.
