use actix_cors::Cors;
use actix_web::{get, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use csv::Reader;
use env_logger::Env;
use log::{error, info, warn};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize)]
struct Question {
    question: String,
    answer: String,
    options: Vec<String>,
    // 添加一个字段来存储选项的原始顺序
    option_mapping: Vec<char>, // 存储ABCD的映射关系
}

#[get("/questions")]
async fn get_random_questions() -> impl Responder {
    info!("Received request for random questions");
    let start_time = Instant::now();

    let file = match File::open("/home/aoba/Downloads/选择题.csv") {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to open CSV file: {}", e);
            return HttpResponse::InternalServerError().json("Failed to load questions");
        }
    };

    let reader = BufReader::new(file);
    let mut csv_reader = Reader::from_reader(reader);

    let mut questions = Vec::new();
    for (index, result) in csv_reader.records().enumerate() {
        match result {
            Ok(record) => {
                let original_answer = record[0].to_string();
                let question = record[1].to_string();

                // 创建选项和答案的映射
                let mut options_with_index: Vec<(String, char)> = vec![
                    (record[2].to_string(), 'A'),
                    (record[3].to_string(), 'B'),
                    (record[4].to_string(), 'C'),
                    (record[5].to_string(), 'D'),
                ];

                // 随机打乱选项顺序
                options_with_index.shuffle(&mut rand::thread_rng());

                // 分离选项和映射
                let options: Vec<String> = options_with_index.iter().map(|(opt, _)| opt.clone()).collect();
                let option_mapping: Vec<char> = options_with_index.iter().map(|(_, ch)| *ch).collect();

                // 找到原始答案对应的新位置
                let original_answer_char = original_answer.chars().next().unwrap();
                let new_answer_position = option_mapping.iter().position(|&x| x == original_answer_char).unwrap();
                let new_answer = (b'A' + new_answer_position as u8) as char;

                info!("Question {}: Original answer: {}, New answer: {}", index + 1, original_answer, new_answer);

                questions.push(Question {
                    question,
                    answer: new_answer.to_string(),
                    options,
                    option_mapping,
                });
            }
            Err(e) => {
                warn!("Error reading record at index {}: {}", index, e);
                continue;
            }
        }
    }

    if questions.is_empty() {
        error!("No questions loaded from CSV");
        return HttpResponse::InternalServerError().json("No questions available");
    }

    // 随机选择题目
    questions.shuffle(&mut rand::thread_rng());
    let selected_questions = questions.into_iter().take(10).collect::<Vec<_>>();

    let duration = start_time.elapsed();
    info!(
        "Successfully processed request. Returned {} questions. Time taken: {:?}",
        selected_questions.len(),
        duration
    );

    HttpResponse::Ok().json(selected_questions)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    info!("Starting quiz server...");

    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %r %s %b %{Referer}i %{User-Agent}i %T"))
            .wrap(cors)
            .service(get_random_questions)
    })
        .bind("127.0.0.1:8790")?
        .run()
        .await
}