use axum::{routing::get, Router};

#[tokio::main] // Define que o projeto usa o motor assíncrono Tokio
async fn main() {
    // 1. Definimos as rotas
    let app = Router::new()
        .route("/", get(|| async { "GUARDIA API: Sistema Ativo" }));

    // 2. Definimos o endereço (localhost:3000)
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    
    println!("🚀 Servidor rodando em http://localhost:3000");

    // 3. Iniciamos o servidor
    axum::serve(listener, app).await.unwrap();
}