mod calendar;
mod client;
mod config;
mod gpa;
mod macros;
mod semester;
mod subject;

use chrono::Datelike;
use clap::{Parser, Subcommand};
use client::LoginError;
use colored::Colorize;
use config::{Config, Login};
use confy::get_configuration_file_path;
use futures::future::join_all;
use gpa::*;
use log::{info, LevelFilter};
use semester::*;
use std::{fs, path::PathBuf, sync::Arc};
use subject::*;
use tabled::{
    settings::{object::Rows, Remove, Style},
    Table,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Display score for each task
    #[arg(short, long)]
    tasks: bool,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Log in to tsinglanstudent.schoolis.cn and store login info
    Login,
    /// Export class schedule to iCalendar format
    #[clap(name = "ical")]
    ICal(ICalArgs),
}

#[derive(Parser)]
struct ICalArgs {
    /// Path to output the ics file
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
    /// Fix class times for high school
    #[arg(long)]
    high_school: bool,
}

#[tokio::main]
async fn main() {
    let config_path = get_configuration_file_path("tls-xb", "config").unwrap();
    if fs::metadata(&config_path).is_err() {
        // if the config file doesn't exist, save the default one
        config::save_config(&Config::default());
    }
    let config = config::get_config();
    let cli = Cli::parse();
    env_logger::Builder::new()
        .filter_level(if cli.verbose {
            LevelFilter::Info
        } else {
            LevelFilter::Warn
        })
        .init();
    let client = Arc::new(match &cli.command {
        Some(Commands::Login) => {
            let mut config = config::login();
            login(&mut config).await
        }
        _ => {
            let login_path = get_configuration_file_path("tls-xb", "login").unwrap();
            let mut login_info = if fs::metadata(&login_path).is_ok() {
                config::get_login()
            } else {
                // if the login file doesn't exist, do tls-xb login.
                config::login()
            };

            login(&mut login_info).await
        }
    });

    info!("Fetching semesters");
    let semesters = get_semesters(&client).await;

    if let Some(Commands::ICal(ical_args)) = &cli.command {
        let semester = get_current_semester(&semesters).unwrap();
        let calendar = calendar::Calendar::new(
            &client,
            semester.start_date.into(),
            semester.end_date.into(),
            ical_args.high_school,
        )
        .await
        .export_ical();

        if let Some(output_path) = &ical_args.output {
            std::fs::write(output_path, calendar.to_string()).expect("Unable to write file");
            info!("Calendar exported to: {}", output_path.display());
        } else {
            println!("{}", calendar);
        }
        std::process::exit(0)
    }

    let semester = select_semester(&semesters);

    info!("Fetching subjects");
    let score_mapping_lists = Arc::new(default_score_mapping_lists());

    let shared_client = Arc::clone(&client);
    let subject_dynamic_scores_handle =
        tokio::spawn(async move { get_subject_dynamic_scores(&shared_client, semester.id).await });

    let shared_client = Arc::clone(&client);
    let elective_class_ids_handle =
        tokio::spawn(async move { get_elective_class_ids(&shared_client).await });

    info!("Fetching GPA");
    let shared_client = Arc::clone(&client);
    let gpa_handle = tokio::spawn(async move { get_gpa(&shared_client, semester.id).await });

    info!("Fetching subject scores");
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
    let subject_dynamic_scores = subject_dynamic_scores_handle.await.unwrap();
    let results = join_all(handles).await;
    for result in results {
        let mut subject = result.unwrap();
        adjust_weights(&mut subject, &elective_class_ids);
        overlay_subject(&mut subject, &subject_dynamic_scores, &score_mapping_lists);
        subjects.push(subject);
    }

    for subject in &subjects {
        print_subject(subject, &cli, &config);
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
        "Calculated Unweighted GPA: {:.2} / {:.2} ({:.1}%)",
        calculated_gpa.unweighted_gpa,
        calculated_gpa.unweighted_max_gpa,
        calculated_gpa.unweighted_gpa / calculated_gpa.unweighted_max_gpa * 100.0
    );
}

fn select_semester(semesters: &[Semester]) -> Semester {
    let mut current_semester = 0;
    for (i, semester) in semesters.iter().enumerate().rev() {
        println!(
            "{:2}: {}-{}, Semester {}",
            i,
            semester.start_date.year(),
            semester.end_date.year(),
            semester.semester,
        );
        if semester.is_now {
            current_semester = i;
        }
    }
    let input = prompt_input!("Choose a semester [{current_semester}]: ");
    if !input.is_empty() {
        current_semester = input.parse().expect("Input not an integer");
    }
    semesters[current_semester].clone()
}

fn round_score(value: f64, decimal_places: u32) -> f64 {
    let multiplier = 10f64.powi(decimal_places as i32);
    (value * multiplier).round() / multiplier
}

fn print_subject(subject: &Subject, cli: &Cli, config: &Config) {
    if subject.total_score.is_nan() {
        return;
    }
    let mut data = vec![(
        colorize(&subject.subject_name, &subject.score_level, &config.colors),
        format!(
            "{}{}",
            round_score(subject.total_score, 1),
            if subject.extra_credit > 0.0 {
                format!(" ({} Extra credit)", round_score(subject.extra_credit, 2))
            } else {
                String::new()
            }
        ),
        subject.score_level.to_string(),
        subject.gpa.to_string(),
        subject.score_mapping_list_id.to_string() + if subject.elective { " Elective" } else { "" },
    )];
    for evaluation_project in &subject.evaluation_projects {
        if evaluation_project.score_is_null {
            continue;
        }
        let row = get_evaluation_project_row(evaluation_project, config);
        data.push(row);
        if cli.tasks {
            let tasks = get_evaluation_project_task_list_row(subject, evaluation_project, config);
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

            let mut row = get_evaluation_project_row(evaluation_project, config);
            row.0.insert_str(0, "- ");
            row.4.insert_str(0, "- ");
            data.push(row);
            if cli.tasks {
                let mut tasks =
                    get_evaluation_project_task_list_row(subject, evaluation_project, config);
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
    config: &Config,
) -> (String, String, String, String, String) {
    (
        colorize(
            &evaluation_project.evaluation_project_e_name,
            &evaluation_project.score_level,
            &config.colors,
        ),
        format!("{}", round_score(evaluation_project.score, 1)),
        evaluation_project.score_level.to_string(),
        evaluation_project.gpa.to_string(),
        format!(
            "{}% ({}%)",
            round_score(evaluation_project.adjusted_proportion, 2),
            round_score(evaluation_project.proportion, 2),
        ),
    )
}

fn get_evaluation_project_task_list_row(
    subject: &Subject,
    evaluation_project: &EvaluationProject,
    config: &Config,
) -> Vec<(String, String, String, String, String)> {
    let mut task_rows = Vec::new();
    let learning_tasks: Vec<&LearningTask> = evaluation_project
        .learning_task_and_exam_list
        .iter()
        .filter(|task| task.score.is_some())
        .collect();
    for learning_task in &learning_tasks {
        let weight = evaluation_project.adjusted_proportion / learning_tasks.len() as f64;
        let score = round_score(
            learning_task.score.unwrap_or(f64::NAN) / learning_task.total_score * 100.0,
            2,
        );
        let row = (
            format!(
                "- {}",
                colorize(
                    &learning_task.name,
                    &score_level_from_score(score, &subject.score_mapping_list),
                    &config.colors
                )
            ),
            format!(
                "{} / {}",
                learning_task.score.unwrap_or(f64::NAN),
                learning_task.total_score
            ),
            format!("{score}%"),
            String::new(),
            format!("- {}%", round_score(weight, 2)),
        );
        task_rows.push(row);
    }
    task_rows
}

fn colorize(string: &str, score_level: &str, color_scheme: &config::ColorScheme) -> String {
    let letter = score_level.chars().next().unwrap();
    let color = match letter {
        'A' => &color_scheme.a_color,
        'B' => &color_scheme.b_color,
        'C' => &color_scheme.c_color,
        'D' => &color_scheme.d_color,
        'F' => &color_scheme.f_color,
        _ => &color_scheme.text_color,
    }
    .as_str();
    if score_level == "A+" || score_level == "F" {
        return string.color(color).bold().to_string();
    }
    string.color(color).to_string()
}

async fn login(config: &mut Login) -> reqwest::Client {
    info!("Logging in");
    let mut client;
    let login_limit = 3;
    for _ in 1..=login_limit {
        client = client::login(config).await;
        match client {
            Ok(client) => {
                config::save_login(config);
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
        config::save_login(config);
        return client;
    }
    panic!("{login_limit} incorrect login attempts.");
}
