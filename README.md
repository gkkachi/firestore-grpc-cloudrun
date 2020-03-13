# firestore_grpc_cloudrun

A gRPC client library for Firestore, intended to run on Cloud Run.

## Usage

Add this to your `Cargo.toml`:

``` toml
[dependencies]
firestore = { version = "0.1", package = "firestore_grpc_cloudrun" }
```

## Examples

### `CreateDocument`

#### Create a new app

```bash
cargo new firestore-rust && cd firestore-rust
```

#### `src/main.rs`

``` rust
use firestore::*;
use futures::try_join;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::convert::Infallible;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([0, 0, 0, 0], get_port()));

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(root))
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn root(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let res = if let Ok(doc) = create_document().await {
        format!("Document: {:?}", doc)
    } else {
        "Failed to get document.".into()
    };
    Ok(Response::new(res.into()))
}

async fn create_document() -> Result<Document, BoxError> {
    let (mut client, project_id) = try_join!(
        get_client(),
        get_project_id(),
    )?;
    let parent = format!("projects/{}/databases/(default)/documents", project_id);
    let collection_id = "greetings".into();
    let document_id = "".into();
    let mut fields = std::collections::HashMap::new();
    fields.insert(
        "message".into(),
        Value {
            value_type: Some(value::ValueType::StringValue(
                "Hello world from CloudRun!".into(),
            )),
        },
    );
    let document = Some(Document {
        name: "".into(),
        fields,
        create_time: None,
        update_time: None,
    });
    let res = client
        .create_document(CreateDocumentRequest {
            parent,
            collection_id,
            document_id,
            document,
            mask: None,
        })
        .await?;
    Ok(res.into_inner())
}

fn get_port() -> u16 {
    std::env::var("PORT")
        .ok()
        .and_then(|x| x.parse().ok())
        .unwrap_or(8080)
}
```

#### `Cargo.toml`

``` toml
[dependencies]
firestore = { version = "*", package = "firestore_grpc_cloudrun" }
futures = "0.3"
hyper = "0.13"
tokio = { version = "0.2", features = ["full"] }
```

#### `Dockerfile`

``` Dockerfile
FROM rust:1-stretch as build-env
WORKDIR /app
ADD . .
RUN rustup update && rustup component add rustfmt && cargo build --release

FROM debian:stretch-slim
WORKDIR /app
RUN apt-get update && apt-get install -y libgcc1 libgomp1 libstdc++6 ca-certificates &&  update-ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=build-env /app/target/release/firestore-rust /app/main
ENTRYPOINT ["./main"]

```
