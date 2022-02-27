# Find Empty Workspace
Finds the unused i3 workspace with the smallest number.
Then, moves the currently focused container to it, and/or focuses the selected workspace.
If there are no empty workspaces left, a notification will be displayed.

This program relies on `notify-send` and `i3-msg`.

```
find-empty-workspace 0.3.0

USAGE:
    find-empty-workspace [FLAGS] <Workspace Names>...

FLAGS:
    -h, --help       Prints help information
    -m, --move       Move the focused container to an empty workspace.
    -s, --stay       Stay on the current workspace.
    -V, --version    Prints version information

ARGS:
    <Workspace Names>...    Names of all possible workspaces. 
```

To call this script you need to specify _all_ possible workspace names.
Ideally you already put those in variables in your `i3/config`.
Here, I'll assume workspaces `$ws1`, ..., `$wsN`.

To switch to an empty workspace.
```
find-empty-workspace $ws1 ... $wsN
```

To move the focused container to an empty workspace but stay on the current workspace.
```
find-empty-workspace -m -s <$ws1 ... $wsN>
```

To move the focused container to an empty workspace and switch to that same workspace.
```
find-empty-workspace -m <$ws1 ... $wsN>
```
