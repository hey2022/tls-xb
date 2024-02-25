mod client;
mod config;
mod gpa;
mod semester;
mod subject;

use clap::{Parser, Subcommand};
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
    let config = get_config();
    let client = client::login(&config).await;

    let semesters = get_semesters(&client).await;
    let semester_id = select_semester(&semesters);

    let score_mapping_lists = default_score_mapping_lists();

    let subject_ids = get_subject_ids(&client, semester_id).await;
    let mut handles = Vec::new();
    let arc_client = Arc::new(client.clone());
    let arc_score_mapping_list = Arc::new(score_mapping_lists.clone());
    for subject_id in subject_ids {
        let client = Arc::clone(&arc_client);
        let score_mapping_lists = Arc::clone(&arc_score_mapping_list);
        let handle = tokio::spawn(async move {
            get_subject(&client, semester_id, subject_id, &score_mapping_lists).await
        });
        handles.push(handle);
    }
    let mut subjects = Vec::new();
    for handle in handles {
        subjects.push(handle.await.unwrap());
    }
    for subject in subjects.iter() {
        print_subject(subject);
    }

    let gpa = get_gpa(&client, semester_id).await;
    let calculated_gpa = calculate_gpa(&subjects);
    println!("GPA: {}", gpa);
    println!("Calculated GPA: {:.2}", calculated_gpa);
}

fn select_semester(semesters: &[Semester]) -> u64 {
    let mut current_semester = 0;
    for (i, semester) in semesters.iter().enumerate().rev() {
        println!("{:2}: {}.{}", i, semester.year, semester.semester);
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
    if input != "\n" {
        current_semester = input.trim().parse().expect("Input not an integer");
    }
    semesters[current_semester].id
}

fn print_subject(subject: &Subject) {
    if subject.total_score.is_nan() {
        return;
    }
    println!(
        "{}: {} / {} / {} ({})",
        subject.subject_name,
        subject.total_score,
        subject.score_level,
        subject.gpa,
        subject.score_mapping_list_id
    );
    for evaluation_project in subject.evaluation_projects.iter() {
        if !evaluation_project.score_is_null {
            println!(
                "{}: {} / {} / {} ({}%)",
                evaluation_project.evaluation_project_e_name,
                evaluation_project.score,
                evaluation_project.score_level,
                evaluation_project.gpa,
                evaluation_project.proportion,
            );
        }
    }
    println!();
}
