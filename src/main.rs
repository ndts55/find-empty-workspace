use i3_ipc::{Connect, I3};
use std::{io, process::Command};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(short = "m", long = "move")]
    move_container: bool,

    #[structopt(short = "f", long = "focus")]
    focus_workspace: bool,

    #[structopt(name = "NAME")]
    names: Vec<String>,
}

fn main() -> io::Result<()> {
    if let Err(error_str) = run() {
        Command::new("notify-send")
            .arg(format!("\"{0}\"", error_str))
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
    // TODO Check whether currently focused workspace is empty.
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

fn find_next_free_workspace(all_names: &Vec<String>, active_names: &Vec<String>) -> Option<String> {
    if active_names.len() == 10 {
        return None;
    }

    all_names
        .iter()
        .find(|&name| !active_names.contains(name))
        .cloned()
}
