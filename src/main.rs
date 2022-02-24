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

    #[structopt(short, long, help = "Stay on the current workspace.")]
    stay: bool,

    #[structopt(
        name = "Workspace Names",
        required = true,
        min_values = 1,
        help = "Names of all possible workspaces."
    )]
    names: Vec<String>,
}

fn main() -> io::Result<()> {
    // Run the actual logic and notify the user about any errors.
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

    let mut i3 = I3::connect().map_err(|_| "Unable to connect to i3")?;
    let active_workspaces = i3
        .get_workspaces()
        .map_err(|_| "Unable to retrieve workspaces.")?;

    is_action_necessary(&mut i3, &active_workspaces)?;

    let free_workspace_name = find_next_free_workspace(&opt.names, &active_workspaces)?;
    i3.run_command(build_command(&opt, &free_workspace_name))
        .map_err(|_| "Unable to execute the command.")?;

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

    let focused_workspace_name = active_workspaces
        .iter()
        .filter(|w| w.focused)
        .next()
        .map(|w| w.name.clone());
    assert!(
        focused_workspace_name.is_some(),
        "There must always be one focused workspace."
    );

    let i3_node_name = Some(String::from("__i3"));
    let content_node_name = Some(String::from("content"));

    let tree: Node = i3.get_tree().expect("Unable to access tree.");
    // Retrieve nodes for workspaces from tree and check if the current workspace is empty, i.e., has no nodes.
    if tree
        .nodes
        .iter()
        .filter(|o| !o.name.eq(&i3_node_name)) // Output nodes only.
        .flat_map(|o| &o.nodes) // Output node children.
        .filter(|c| c.name.eq(&content_node_name)) // Each output node has a content node.
        .flat_map(|c| &c.nodes) // Content node children are workspaces.
        .any(|w: &Node| w.name.eq(&focused_workspace_name) && w.nodes.len() == 0)
    {
        Err("The current workspace is empty.")
    } else {
        // Action is necessary. Return Ok.
        Ok(())
    }
}

fn find_next_free_workspace(
    all_names: &Vec<String>,
    active_workspaces: &Vec<Workspace>,
) -> Result<String, &'static str> {
    let active_names: Vec<&String> = active_workspaces.iter().map(|ws| &ws.name).collect();
    all_names
        .iter()
        .find(|name| !active_names.contains(name))
        .cloned()
        .ok_or("No empty workspaces available.")
}

fn build_command(opt: &Opt, workspace_name: &String) -> String {
    let mut command = String::new();

    if opt.move_container {
        command.push_str(&format!("move container to workspace {0};", workspace_name));
    }

    if !opt.stay {
        command.push_str(&format!("workspace {0};", workspace_name));
    }

    command
}
