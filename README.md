# find_empty_workspace
Finds the unused i3 workspace with the smallest number.
Then, moves the currently focused container to it, and/or focuses the selected workspace.
If there are no empty workspaces left a notification will be displayed.

This programm relies on `notify-send` and `i3-msg`.

```
find_empty_workspace 0.1.0

USAGE:
    find_empty_workspace [FLAGS] <NAMES>...

FLAGS:
    -f, --focus      Focus an empty workspace.
    -h, --help       Prints help information
    -m, --move       Move the focused container to an empty workspace.
    -V, --version    Prints version information

ARGS:
    <NAMES>...    
```

To call this script you need to specify _all_ possible workspace names.
Ideally you already put those in variables in your `i3/config`.
Then you can call `find_empty_workspace` like:

```
~/.config/find_empty_workspace --move --focus $ws1 $ws2 ... $wsN
```

