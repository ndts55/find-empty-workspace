use i3_ipc::{
    reply::{Node, Workspace},
    Connect, I3Stream, I3,
};
use std::{io, process::Command};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "find_empty_workspace")]
struct Opt {
    #[structopt(
        short = "m",
        long = "move",
        help = "Move the focused container to an empty workspace."
    )]
    move_container: bool,

    #[structopt(short = "f", long = "focus", help = "Focus an empty workspace.")]
    focus_workspace: bool,

    #[structopt(name = "NAMES", required = true, min_values = 1)]
    names: Vec<String>,
}

fn main() -> io::Result<()> {
    if let Err(error_str) = run() {
        Command::new("notify-send")
            .arg(format!("{0}", error_str))
            .spawn()
            .expect("notify-send is missing");
    }
    Ok(())
}

fn run() -> Result<(), &'static str> {
    let opt = Opt::from_args();
    if !opt.move_container && !opt.focus_workspace {
        return Ok(());
    }

    let mut i3 = I3::connect().map_err(|_| "Unable to connect to i3")?;
    let active_workspaces = i3
        .get_workspaces()
        .map_err(|_| "Unable to retrieve workspaces.")?;

    is_action_necessary(&mut i3, &active_workspaces)?;

    let active_names = active_workspaces.into_iter().map(|ws| ws.name).collect();
    let free_workspace_name = find_next_free_workspace(&opt.names, &active_names)
        .ok_or("No empty workspaces available.")?;

    let mut command = String::new();

    if opt.move_container {
        let move_command = format!("move container to workspace {0};", free_workspace_name);
        command.push_str(&move_command);
    }

    if opt.focus_workspace {
        let focus_command = format!("workspace {0};", free_workspace_name);
        command.push_str(&focus_command);
    }

    i3.run_command(command)
        .map_err(|_| "Unable to execute the command")?;

    Ok(())
}

fn is_action_necessary(
    i3: &mut I3Stream,
    active_workspaces: &Vec<Workspace>,
) -> Result<(), &'static str> {
    // Action is necessary iff current workspace is not empty and there is at least one inactive workspace.
    if active_workspaces.len() == 10 {
        return Err("No empty workspaces available.");
    }

    let focused_workspace_name = Some(
        active_workspaces
            .iter()
            .filter(|w| w.focused)
            .next()
            .expect("There is always one focused workspace")
            .name
            .clone(),
    );

    let i3_node_name = Some(String::from("__i3"));
    let content_node_name = Some(String::from("content"));

    let tree: Node = i3.get_tree().map_err(|_| "Unable to access tree.")?;
    if tree
        .nodes
        .iter()
        .filter(|o| !o.name.eq(&i3_node_name))
        .flat_map(|o| &o.nodes)
        .filter(|c| c.name.eq(&content_node_name))
        .flat_map(|c| &c.nodes)
        .any(|w: &Node| w.name.eq(&focused_workspace_name) && w.nodes.len() == 0)
    {
        Err("The current workspace is empty.")
    } else {
        Ok(())
    }
}

fn find_next_free_workspace(all_names: &Vec<String>, active_names: &Vec<String>) -> Option<String> {
    all_names
        .iter()
        .find(|&name| !active_names.contains(name))
        .cloned()
}
