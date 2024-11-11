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

## The TATATA language

The TATATA language is heavily inspired by the [Portal 2 TAS language](https://wiki.portal2.sr/TASing). Every line
consists of a timestamp and an action field, separated by an angle bracket (`>`). The timestamp is the point in time to
execute the associated actions, in milliseconds after the script was started. The action field can contain any number of
actions separated by a semicolon (`;`). Valid actions are:

- `mousemove`: Move the mouse to the specified absolute position
- `mousepress`: Press and immediately release the specified mouse button:
  - `1`: Left click
  - `2`: Right click
  - `3`: Middle click
  - `4`: Back
  - `5`: Forward
- `keypress`: Press and immediately release a key on the keyboard. This can take any of the following:
  - A letter or number found on a standard keyboard
  - A symbol you can type on the base layer of your keyboard, i.e. without holding any modifiers
  - The following function keys: `f1`, `f2`, `f3`, `f4`, `f5`, `f6`, `f7`, `f8`, `f9`, `f10`, `f11`, `f12`, `f13`,
  `f14`, `f15`, `f16`, `f17`, `f18`, `f19`, `f20`
  - The following modifiers: `control`, `shift`, `alt`, `super` (windows/command key), `capslock`
  - Arrow keys: `up`, `down`, `left`, `right`
  - The following other special keys: `tab`, `escape`, `space`, `enter`, `backspace`, `insert`, `delete`, `home`, `end`,
  `pageup`, `pagedown`

### Example

```
0>mousemove 500 100; mousepress 1
100>keypress h;keypress e;keypress l;keypress l;keypress o
200>mousemove 800 100
250>mousepress 1

// Lines starting with a double slash are considered comments
// You cannot currently place comments at the end of action lines
```

## Future plans

TATATA is obviously still in super early development. Future plans include:

- [ ] Text actions (i.e. typing out the specified text without needing a ton of `keypress` events)
- [ ] Interpolated mouse movements (i.e. moving the mouse over time as opposed to an instant "snap")
- [ ] Explicitly specified _down_ and _up_ presses of mouse and keyboard buttons
- [ ] Relative timestamps (i.e. specifying that one line happens `x` milliseconds after the previous one instead of at an
absolute time)
- [ ] Support for inline comments and multiline comments
- [ ] Meaningful parser warnings and comments upon running a script
- [ ] Syntax highlighting extensions/plugins for major code editors
- [ ] A graphical editor and event viewer
- [ ] A system tray icon for running scripts

> [!CAUTION]
> Due to us still being really early into TATATA's development, expect many upcoming changes to break existing scripts.
> If this is a concern, you can keep older versions of TATATA on your computer for usage with older scripts.
> Alternatively, you can download older releases from GitHub as I intend on keeping the version history intact.
