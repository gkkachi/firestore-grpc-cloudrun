const URL: &'static str = "https://firestore.googleapis.com";
const DOMAIN: &'static str = "firestore.googleapis.com";

pub mod compute_metadata;
pub type BoxError = Box<dyn std::error::Error + Sync + Send + 'static>;
pub use compute_metadata::get_project_id;
use firestore_client::FirestoreClient;
use firestore_grpc::tonic::{
    metadata::MetadataValue,
    transport::{Channel, ClientTlsConfig},
    Request,
};
pub use firestore_grpc::v1::*;
use futures::{future::TryFutureExt, try_join};

pub async fn get_client() -> Result<FirestoreClient<Channel>, BoxError> {
    let endpoint = Channel::from_static(URL).tls_config(ClientTlsConfig::new().domain_name(DOMAIN));

    let (channel, token) = try_join!(
        endpoint.connect().map_err(|e| BoxError::from(e)),
        compute_metadata::get_token()
    )?;

    let bearer_token = format!("Bearer {}", token);
    let header_value = MetadataValue::from_str(&bearer_token)?;

    let service = FirestoreClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut()
            .insert("authorization", header_value.clone());
        Ok(req)
    });
    Ok(service)
}
