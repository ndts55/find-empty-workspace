use eyre::eyre;
use i3_ipc::{
    reply::{Node, Workspace},
    Connect, I3Stream, I3,
};
use std::{io, process::Command};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "find-empty-workspace")]
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
        short,
        long,
        help = "Force switching workspaces even if the current one is empty."
    )]
    force: bool,

    #[structopt(
        name = "Workspace Names",
        required = true,
        min_values = 1,
        help = "Names of all possible workspaces sorted by index."
    )]
    names: Vec<String>,
}

fn main() {
    // Run the actual logic and notify the user about any errors.
    if let Err(error_str) = run() {
        Command::new("notify-send")
            .arg(format!("{0}", error_str))
            .spawn()
            .expect("notify-send is missing");
    }
}

fn run() -> eyre::Result<()> {
    let opt = Opt::from_args();

    let mut i3 = I3::connect().map_err(|e| eyre!("Unable to connect to i3\n{}", e.to_string()))?;
    let active_workspaces = i3
        .get_workspaces()
        .map_err(|e| eyre!("Unable to retrieve workspaces.\n{}", e.to_string()))?;

    // There is nothing we CAN do if there are no empty workspaces available.
    is_inactive_workspace_available(&active_workspaces, &opt.names)?;
    let tree: Node = i3
        .get_tree()
        .map_err(|e| eyre!("Unable to access tree.\n{}", e.to_string()))?;
    let workspace_name = determine_desired_workspace_name(&tree, &opt, &active_workspaces)?;
    switch_to_workspace(&mut i3, &opt, workspace_name)
}

fn is_inactive_workspace_available(
    active_workspaces: &Vec<Workspace>,
    names: &Vec<String>,
) -> eyre::Result<()> {
    // Check whether there is at least one inactive workspace.
    if active_workspaces.len() == names.len() {
        Err(eyre!("No empty workspaces available."))
    } else {
        Ok(())
    }
}

fn determine_desired_workspace_name(
    tree: &Node,
    opt: &Opt,
    active_workspaces: &Vec<Workspace>,
) -> eyre::Result<String> {
    let current_workspace = extract_current_workspace(&active_workspaces)?;
    let smallest_inactive_workspace_name =
        find_smallest_inactive_workspace_name(&opt.names, &active_workspaces)?;

    if is_workspace_empty(&tree, &current_workspace)? {
        if opt.force {
            let current_index_opt = opt.names.iter().position(|n| n.eq(&current_workspace.name));
            let smallest_index = opt
                .names
                .iter()
                .position(|n| n.eq(smallest_inactive_workspace_name))
                .ok_or(eyre!("Smallest index is not in names?"))?;

            Ok(match current_index_opt {
                None => smallest_inactive_workspace_name.clone(),
                Some(current_index) if current_index < smallest_index => {
                    return Err(eyre!("This is already the smallest workspace."))
                }
                Some(current_index) if current_index > smallest_index => {
                    smallest_inactive_workspace_name.clone()
                }
                Some(_) => return Err(eyre!("This should never happen.")),
            })
        } else {
            Err(eyre!("The current workspace is empty."))
        }
    } else {
        Ok(smallest_inactive_workspace_name.clone())
    }
}

fn extract_current_workspace(active_workspaces: &Vec<Workspace>) -> eyre::Result<&Workspace> {
    active_workspaces
        .iter()
        .filter(|w| w.focused)
        .next()
        .ok_or(eyre!("There must always be one focused workspace."))
}

fn is_workspace_empty(tree: &Node, workspace: &Workspace) -> eyre::Result<bool> {
    let i3_node_name = Some(String::from("__i3"));
    let content_node_name = Some(String::from("content"));
    let workspace_name_opt = Some(workspace.name.clone());
    // Retrieve nodes for workspaces from tree and check whether the given workspace is empty, i.e., has no nodes.
    // TODO Check floating nodes as well.
    Ok(tree
        .nodes
        .iter()
        .filter(|o| !o.name.eq(&i3_node_name)) // Output nodes only.
        .flat_map(|o| &o.nodes) // Output node children.
        .filter(|c| c.name.eq(&content_node_name)) // Each output node has a content node.
        .flat_map(|c| &c.nodes) // Content node children are workspaces.
        .any(|w: &Node| w.name.eq(&workspace_name_opt) && w.nodes.len() == 0))
}

fn find_smallest_inactive_workspace_name<'a, 'b>(
    names: &'a Vec<String>,
    active_workspaces: &'b Vec<Workspace>,
) -> eyre::Result<&'a String> {
    let active_names: Vec<&String> = active_workspaces.iter().map(|ws| &ws.name).collect();
    names
        .iter()
        .find(|name| !active_names.contains(name))
        .ok_or(eyre!("No empty workspaces available."))
}

fn switch_to_workspace(i3: &mut I3Stream, opt: &Opt, workspace_name: String) -> eyre::Result<()> {
    let rc = i3.run_command(build_command(&opt, &workspace_name));
    rc.and_then(|ss| {
        ss.iter()
            .filter_map(|s| if !s.success { s.error.clone() } else { None })
            .reduce(|acc, s| format!("{}\n{}", acc, s))
            .map_or(Ok(()), |s| Err(io::Error::new(io::ErrorKind::Other, s)))
    })
    .map_err(|e| eyre!("Unable to execute command.\n{}", e.to_string()))
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
