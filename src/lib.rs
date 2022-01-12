mod data;
mod error;

use axum::{
    routing::get,
    Router,
};
use data::user;

pub async fn run_server() {
    // build our application with a single route
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    let db_uri = "postgres://dodeka:postpost@localhost:3141/dodeka";

    let dsrc = data::source::Source::new(db_uri).await.unwrap();
    let mut u = user::get_user_by_id(&dsrc, 1).await.unwrap();
    println!("{:?}", u);
    //let i = user::new_user(&dsrc, u).await.unwrap();
    //println!("{}", i);
    //u.password_file = "abc".to_owned();
    //user::upsert_user_row(&dsrc, u).await.unwrap();
    // let u = user::get_user_by_usph(&dsrc, "fakerecord").await.unwrap();
    // println!("{:?}", u);

    // run it with hyper on localhost:3000
    axum::Server::bind(&"127.0.0.1:3031".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}