mod client;
mod config;
mod gpa;
mod semester;
mod subject;

use clap::{Parser, Subcommand};
use colored::{ColoredString, Colorize};
use config::*;
use gpa::{calculate_gpa, default_score_mapping_lists, get_gpa};
use semester::*;
use std::io::Write;
use std::sync::Arc;
use subject::*;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Login,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Some(command) = cli.command {
        match command {
            Commands::Login => {
                login();
                std::process::exit(0);
            }
        }
    }

    println!(
        ":: Getting config.toml from {}...",
        confy::get_configuration_file_path("tls-xb", "config")
            .unwrap()
            .to_str()
            .unwrap()
    );
    let config = get_config();

    println!(":: Logging in...");
    let client = client::login(&config).await;

    println!(":: Fetching semesters...");
    let semesters = get_semesters(&client).await;
    let semester = select_semester(&semesters);

    let score_mapping_lists = default_score_mapping_lists();

    println!(":: Fetching subjects...");
    let subject_ids = get_subject_ids(&client, semester.id).await;
    let elective_class_ids = get_elective_class_ids(&client, semester.start_date).await;

    println!(":: Fetching subject scores...");
    let mut handles = Vec::new();
    let arc_client = Arc::new(client.clone());
    let arc_score_mapping_list = Arc::new(score_mapping_lists.clone());
    let arc_elective_class_ids = Arc::new(elective_class_ids.clone());
    for subject_id in subject_ids {
        let client = Arc::clone(&arc_client);
        let score_mapping_lists = Arc::clone(&arc_score_mapping_list);
        let elective_class_ids = Arc::clone(&arc_elective_class_ids);
        let handle = tokio::spawn(async move {
            get_subject(
                &client,
                semester.id,
                subject_id,
                &score_mapping_lists,
                &elective_class_ids,
            )
            .await
        });
        handles.push(handle);
    }
    let mut subjects = Vec::new();
    for handle in handles {
        let subject = handle.await.unwrap();
        subjects.push(subject);
    }
    for subject in subjects.iter() {
        print_subject(subject);
    }

    println!(":: Fetching GPA...");
    let gpa = get_gpa(&client, semester.id).await;
    let calculated_gpa = calculate_gpa(&subjects);
    println!("GPA: {}", gpa);
    println!("Calculated GPA: {:.2}", calculated_gpa);
}

fn select_semester(semesters: &[Semester]) -> Semester {
    let mut current_semester = 0;
    for (i, semester) in semesters.iter().enumerate().rev() {
        println!(
            "{:2}: Semester {}, {}-{}",
            i,
            semester.semester,
            semester.year,
            semester.year + 1,
        );
        if semester.is_now {
            current_semester = i;
        }
    }
    print!("Choose a semester [{}]: ", current_semester);
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input = input.trim().to_string();
    if !input.is_empty() {
        current_semester = input.parse().expect("Input not an integer");
    }
    semesters[current_semester].clone()
}

fn print_subject(subject: &Subject) {
    if subject.total_score.is_nan() {
        return;
    }
    println!(
        "{}: {:.1} / {} / {} ({}{})",
        colorize(&subject.subject_name, &subject.score_level),
        subject.total_score,
        subject.score_level,
        subject.gpa,
        subject.score_mapping_list_id,
        if subject.elective { " Elective" } else { "" },
    );
    for evaluation_project in subject.evaluation_projects.iter() {
        if !evaluation_project.score_is_null {
            print_evaluation_project(evaluation_project);
        }
        if !evaluation_project.evaluation_project_list.is_empty() {
            for evaluation_project in evaluation_project.evaluation_project_list.iter() {
                if !evaluation_project.score_is_null {
                    print!("- ");
                    print_evaluation_project(evaluation_project);
                }
            }
        }
    }
    println!();
}

fn print_evaluation_project(evaluation_project: &EvaluationProject) {
    println!(
        "{}: {:.1} / {} / {} ({}%)",
        colorize(
            &evaluation_project.evaluation_project_e_name,
            &evaluation_project.score_level
        ),
        evaluation_project.score,
        evaluation_project.score_level,
        evaluation_project.gpa,
        evaluation_project.proportion,
    );
}

fn colorize(string: &str, score_level: &str) -> ColoredString {
    let letter = score_level.chars().next().unwrap();
    let color = match letter {
        'A' => "green",
        'B' => "blue",
        'C' => "yellow",
        'D' => "red",
        'F' => "red",
        _ => "white",
    };
    if score_level == "A+" || score_level == "F" {
        return string.color(color).bold();
    }
    string.color(color)
}
