mod client;
mod config;
mod gpa;
mod semester;
mod subject;

use clap::{Parser, Subcommand};
use client::LoginError;
use colored::Colorize;
use config::Config;
use futures::future::join_all;
use gpa::*;
use semester::*;
use std::sync::Arc;
use subject::*;
use tabled::{
    settings::{object::Rows, Remove, Style},
    Table,
};
use text_io::read;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(short, long)]
    tasks: bool,
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
    let mut config;
    if let Some(command) = &cli.command {
        match command {
            Commands::Login => {
                config = config::login();
                login(&mut config).await;
            }
        }
    }
    config = config::get_config();
    let client = login(&mut config).await;
    let client = Arc::new(client);

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

    for subject in &subjects {
        print_subject(subject, &cli);
    }

    let gpa = gpa_handle.await.unwrap();
    let calculated_gpa = calculate_gpa(&subjects);
    if gpa.is_nan() {
        println!("GPA: Unreleased");
    } else {
        println!("GPA: {gpa}");
    }
    println!(
        "Calculated GPA: {:.2} / {:.2} ({:.1}%)",
        calculated_gpa.weighted_gpa,
        calculated_gpa.max_gpa,
        calculated_gpa.weighted_gpa / calculated_gpa.max_gpa * 100.0
    );
    println!(
        "Calculated Unweighted GPA: {:.2}",
        calculated_gpa.unweighted_gpa
    );
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

fn print_subject(subject: &Subject, cli: &Cli) {
    if subject.total_score.is_nan() {
        return;
    }
    let mut data = vec![(
        colorize(&subject.subject_name, &subject.score_level),
        format!("{}", (subject.total_score*10.0).round()/10.0),
        subject.score_level.to_string(),
        subject.gpa.to_string(),
        subject.score_mapping_list_id.to_string() + if subject.elective { " Elective" } else { "" },
    )];
    for evaluation_project in &subject.evaluation_projects {
        if evaluation_project.score_is_null {
            continue;
        }
        let row = get_evaluation_project_row(evaluation_project);
        data.push(row);
        if cli.tasks {
            let tasks = get_evaluation_project_task_list_row(evaluation_project);
            for task in tasks {
                data.push(task);
            }
        }

        if evaluation_project.evaluation_project_list.is_empty() {
            continue;
        }
        for evaluation_project in &evaluation_project.evaluation_project_list {
            if evaluation_project.score_is_null {
                continue;
            }

            let mut row = get_evaluation_project_row(evaluation_project);
            row.0.insert_str(0, "- ");
            row.4.insert_str(0, "- ");
            data.push(row);
            if cli.tasks {
                let mut tasks = get_evaluation_project_task_list_row(evaluation_project);
                for task in &mut tasks {
                    task.0.insert(0, '-');
                    task.4.insert(0, '-');
                    data.push(task.clone());
                }
            }
        }
    }
    let table = Table::new(data)
        .with(Remove::row(Rows::first()))
        .with(Style::rounded())
        .to_string();
    println!("{table}");
}

fn get_evaluation_project_row(
    evaluation_project: &EvaluationProject,
) -> (String, String, String, String, String) {
    (
        colorize(
            &evaluation_project.evaluation_project_e_name,
            &evaluation_project.score_level,
        ),
        format!("{}", (evaluation_project.score*10.0).round()/10.0),
        evaluation_project.score_level.to_string(),
        evaluation_project.gpa.to_string(),
        format!(
            "{}% ({}%)",
            (evaluation_project.adjusted_proportion*100.0).round()/100.0, (evaluation_project.proportion*100.0).round()/100.0
        ),
    )
}

fn get_evaluation_project_task_list_row(
    evaluation_project: &EvaluationProject,
) -> Vec<(String, String, String, String, String)> {
    let mut task_rows = Vec::new();
    let learning_tasks: Vec<&LearningTask> = evaluation_project
        .learning_task_and_exam_list
        .iter()
        .filter(|task| task.score.is_some())
        .collect();
    for learning_task in &learning_tasks {
        let weight = evaluation_project.adjusted_proportion / learning_tasks.len() as f64;
        let row = (
            format!("- {}", learning_task.name),
            format!(
                "{} / {}",
                learning_task.score.unwrap_or(f64::NAN),
                learning_task.total_score
            ),
            format!(
                "{}%",
                (learning_task.score.unwrap_or(f64::NAN) / learning_task.total_score * 100.0 * 100.0).round()/100.0
            ),
            String::new(),
            format!("- {}%", (weight*100.0).round()/100.0),
        );
        task_rows.push(row);
    }
    task_rows
}

fn colorize(string: &str, score_level: &str) -> String {
    let letter = score_level.chars().next().unwrap();
    let color = match letter {
        'A' => "green",
        'B' => "blue",
        'C' => "yellow",
        'D' | 'F' => "red",
        _ => "white",
    };
    if score_level == "A+" || score_level == "F" {
        return string.color(color).bold().to_string();
    }
    string.color(color).to_string()
}

async fn login(config: &mut Config) -> reqwest::Client {
    println!(
        ":: Getting config.toml from {}...",
        confy::get_configuration_file_path("tls-xb", "config")
            .unwrap()
            .to_str()
            .unwrap()
    );

    println!(":: Logging in...");
    let mut client;
    let login_limit = 3;
    for _ in 1..=login_limit {
        client = client::login(config).await;
        match client {
            Ok(client) => {
                config::save_config(config);
                println!("Successfully logined and saved.");
                return client;
            }
            Err(LoginError::IncorrectLogin(msg)) => {
                println!("{msg}");
                println!("Sorry, try again.");
                *config = config::login();
            }
            Err(LoginError::ErrorCode((msg, state))) => {
                println!("{msg}");
                println!("Unknown error with code {state}, trying again.");
            }
            Err(LoginError::IncorrectCaptcha(msg)) => {
                println!("{msg}");
                println!("Sorry, wrong captcha, try again.");
            }
        }
    }
    if let Ok(client) = client::login(config).await {
        config::save_config(config);
        println!("Successfully logined and saved.");
        return client;
    }
    panic!("{login_limit} incorrect login attempts.");
}
