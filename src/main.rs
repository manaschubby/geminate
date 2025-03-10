use crossterm::style::Stylize;
use dotenvy::dotenv;
use gemini_rs::Conversation;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::{env, io};
use termimad::crossterm::style::Color::*;
use termimad::*;
use tokio::main;
use uuid::Uuid;

static CONV_DIR: LazyLock<PathBuf, fn() -> PathBuf> = LazyLock::new(|| {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("gemini-chat/convos")
});

fn display_logo() {
    println!("{}", "\n");
    println!("    ██████╗ ███████╗███╗   ███╗██╗███╗   ██╗ █████╗ ████████╗███████╗");
    println!("   ██╔════╝ ██╔════╝████╗ ████║██║████╗  ██║██╔══██╗╚══██╔══╝██╔════╝");
    println!("   ██║  ███╗█████╗  ██╔████╔██║██║██╔██╗ ██║███████║   ██║   █████╗  ");
    println!("   ██║   ██║██╔══╝  ██║╚██╔╝██║██║██║╚██╗██║██╔══██║   ██║   ██╔══╝  ");
    println!("   ╚██████╔╝███████╗██║ ╚═╝ ██║██║██║ ╚████║██║  ██║   ██║   ███████╗");
    println!("    ╚═════╝ ╚══════╝╚═╝     ╚═╝╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝   ╚═╝   ╚══════╝");
    println!("{}", "\n");
}

fn display_user_message(message: &str) {
    let width = terminal_size().0.min(100);
    let mut output = String::new();

    output.push_str("\n╭");
    output.push_str(&"─".repeat(width - 2));
    output.push_str("╮\n");

    let wrapped_text = textwrap::fill(message, width - 4);
    for line in wrapped_text.lines() {
        output.push_str("│ ");
        output.push_str(line);
        output.push_str(&" ".repeat(width - 4 - line.len()));
        output.push_str(" │\n");
    }

    output.push_str("╰");
    output.push_str(&"─".repeat(width - 2));
    output.push_str("╯\n");

    // blue
    println!("{}", output.with(rgb(0, 120, 255)));
}

fn display_ai_message(message: &str) {
    let width = terminal_size().0.min(100);
    let mut output = String::new();

    output.push_str("\n╭");
    output.push_str(&"─".repeat(width - 2));
    output.push_str("╮\n");

    let wrapped_text = textwrap::fill(message, width - 4);
    for line in wrapped_text.lines() {
        output.push_str("│ ");
        output.push_str(line);
        output.push_str(&" ".repeat(width - 4 - line.len()));
        output.push_str(" │\n");
    }

    output.push_str("╰");
    output.push_str(&"─".repeat(width - 2));
    output.push_str("╯\n");

    // yellow
    println!("{}", output.with(rgb(255, 187, 0)));
}

fn terminal_size() -> (usize, usize) {
    match termimad::crossterm::terminal::size() {
        Ok((w, h)) => (w as usize, h as usize),
        Err(_) => (80, 24),
    }
}

fn list_files_in_dir(dir: &Path) -> Option<Vec<PathBuf>> {
    match fs::read_dir(dir) {
        Ok(entries) => {
            let old_convos: Vec<_> = entries
                .flatten()
                .filter_map(|entry| {
                    let path = entry.path();
                    if path.is_file() {
                        Some(path)
                    } else {
                        None
                    }
                })
                .collect();
            (!old_convos.is_empty()).then_some(old_convos)
        }
        Err(e) => {
            eprintln!("Failed to read directory: {}", e);
            None
        }
    }
}

fn prompt_for_conv(skin: &MadSkin) -> bool {
    loop {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        let input = input.trim().to_lowercase();

        match input.as_str() {
            "y" | "" => return true,
            "n" => return false,
            _ => skin.print_text("Invalid input. Please enter 'Y' or 'N'."),
        }
    }
}

fn load_old_conversation(skin: &mut MadSkin, convo: &mut Conversation, conv_uuid: &mut Uuid) {
    match list_files_in_dir(CONV_DIR.as_path()) {
        None => {
            skin.print_text(
                format!("No old convos found in {}", CONV_DIR.as_path().display()).as_str(),
            );
            skin.print_text("Creating new conversation...");
        }
        Some(old_convos) => {
            skin.print_text("Enter valid index to choose convo: ");

            for (index, convo) in old_convos.iter().enumerate() {
                skin.print_text(format!("[{}], {}", index, convo.to_str().unwrap()).as_str());
            }

            loop {
                let mut pick = String::new();
                io::stdin()
                    .read_line(&mut pick)
                    .expect("Failed to read input");

                match pick.trim().parse::<usize>() {
                    Ok(pick) => {
                        if pick < old_convos.len() {
                            skin.print_text(
                                format!("You selected: {}", old_convos[pick].to_str().unwrap())
                                    .as_str(),
                            );
                            convo.load(old_convos[pick].to_str().unwrap());
                            let file_name = old_convos[pick].file_name().unwrap().to_str().unwrap();

                            *conv_uuid = file_name
                                .strip_prefix("convo-")
                                .unwrap()
                                .strip_suffix(".txt")
                                .unwrap()
                                .parse()
                                .unwrap();
                            break;
                        } else {
                            skin.print_text("Invalid index. Try again!");
                        }
                    }
                    Err(_) => {
                        skin.print_text("Invalid input. Please enter a number.");
                    }
                }
            }
        }
    }
}

#[main]
async fn main() {
    dotenv().ok();
    fs::create_dir_all(CONV_DIR.as_path()).unwrap();

    display_logo();

    let mut skin = MadSkin::default();
    skin.set_headers_fg(rgb(255, 187, 0));
    skin.bold.set_fg(Yellow);
    skin.italic.set_fgbg(Magenta, rgb(30, 30, 40));
    skin.bullet = StyledChar::from_fg_char(Yellow, '⟡');
    skin.quote_mark.set_fg(Yellow);
    skin.set_global_bg(rgb(61, 74, 79));

    let mut convo = Conversation::new(
        env::var("GEMINIAI_API").expect("Gemini API not set"),
        "gemini-1.5-flash".to_string(),
    );

    skin.print_text("Would you like to create a new conversation? (Y/n)");

    let mut conv_uuid: Uuid = Uuid::new_v4();
    if prompt_for_conv(&skin) {
        skin.print_text("Starting a new conversation...");
    } else {
        skin.print_text("Continuing with existing conversations...");
        load_old_conversation(&mut skin, &mut convo, &mut conv_uuid);
    }

    display_ai_message("Hi👋 I'm Gemini. How can I help you today? (type 'exit' to leave)");

    loop {
        let mut user_input = String::new();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read input");

        let user_input = user_input.trim();

        if user_input.to_lowercase() == "exit" {
            break;
        }

        display_user_message(user_input);

        let ai_response = convo.prompt(user_input).await;

        display_ai_message(&ai_response);
    }

    let conv_path = CONV_DIR.join(format!("convo-{}.txt", conv_uuid));

    let path = conv_path.to_str().unwrap();

    convo.save(path);

    skin.print_text(format!("Conversation saved in: {}", path).as_str());
}
