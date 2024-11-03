mod client;
mod config;
mod gpa;
mod semester;
mod subject;

use clap::{Parser, Subcommand};
use colored::Colorize;
use config::*;
use futures::future::join_all;
use gpa::*;
use semester::*;
use std::sync::Arc;
use subject::*;
use tabled::{
    settings::{object::Rows, Disable, Style},
    Table,
};
use text_io::read;

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
    let client = Arc::new(client::login(&config).await);

    println!(":: Fetching semesters...");
    let semesters = get_semesters(&client).await;
    let semester = select_semester(&semesters);

    println!(":: Fetching subjects...");
    let score_mapping_lists = Arc::new(default_score_mapping_lists());
    let shared_client = Arc::clone(&client);
    let elective_class_ids_handle =
        tokio::spawn(
            async move { get_elective_class_ids(&shared_client, semester.start_date).await },
        );

    println!(":: Fetching GPA...");
    let shared_client = Arc::clone(&client);
    let gpa_handle = tokio::spawn(async move { get_gpa(&shared_client, semester.id).await });

    println!(":: Fetching subject scores...");
    let subject_ids = get_subject_ids(&client, semester.id).await;
    let mut handles = Vec::new();
    for subject_id in subject_ids {
        let client = Arc::clone(&client);
        let score_mapping_lists = Arc::clone(&score_mapping_lists);
        let handle = tokio::spawn(async move {
            get_subject(&client, semester.id, subject_id, &score_mapping_lists).await
        });
        handles.push(handle);
    }

    let mut subjects = Vec::new();
    let elective_class_ids = elective_class_ids_handle.await.unwrap();
    let results = join_all(handles).await;
    for result in results {
        let mut subject = result.unwrap();
        adjust_weights(&mut subject, &elective_class_ids);
        subjects.push(subject);
    }

    for subject in subjects.iter() {
        print_subject(subject);
    }

    let gpa = gpa_handle.await.unwrap();
    let calculated_gpa = calculate_gpa(&subjects);
    println!("GPA: {gpa}");
    println!("Calculated GPA: {:.2}", calculated_gpa.weighted_gpa);
    println!(
        "Calculated Unweighted GPA: {:.2}",
        calculated_gpa.unweighted_gpa
    );
    println!("GPA Delta: {:.2}", calculated_gpa.gpa_delta);
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
    print!("Choose a semester [{current_semester}]: ");
    let mut input: String = read!("{}\n");
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
    let mut data = vec![(
        colorize(&subject.subject_name, &subject.score_level),
        format!("{:.1}", subject.total_score),
        &subject.score_level,
        subject.gpa,
        subject.score_mapping_list_id.to_string() + if subject.elective { " Elective" } else { "" },
    )];
    for evaluation_project in subject.evaluation_projects.iter() {
        if evaluation_project.score_is_null {
            continue;
        }
        let row = get_evaluation_project_row(evaluation_project);
        data.push(row);
        if evaluation_project.evaluation_project_list.is_empty() {
            continue;
        }
        for evaluation_project in evaluation_project.evaluation_project_list.iter() {
            if !evaluation_project.score_is_null {
                let mut row = get_evaluation_project_row(evaluation_project);
                row.0.insert_str(0, "- ");
                data.push(row);
            }
        }
    }
    let table = Table::new(data)
        .with(Disable::row(Rows::first()))
        .with(Style::rounded())
        .to_string();
    println!("{table}");
}

fn get_evaluation_project_row(
    evaluation_project: &EvaluationProject,
) -> (String, String, &String, f64, String) {
    (
        colorize(
            &evaluation_project.evaluation_project_e_name,
            &evaluation_project.score_level,
        ),
        format!("{:.1}", evaluation_project.score),
        &evaluation_project.score_level,
        evaluation_project.gpa,
        format!("{}%", evaluation_project.proportion),
    )
}

fn colorize(string: &str, score_level: &str) -> String {
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
        return string.color(color).bold().to_string();
    }
    string.color(color).to_string()
}
