use crate::{Coordinator, CoordinatorError};

use rocket::{http::Status, State};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use serde_diff::SerdeDiff;

#[derive(Debug, Serialize, Deserialize, SerdeDiff)]
pub struct InnerLockResponse {
    chunk_id: u64,
    locked: bool,
}

#[derive(Debug, Serialize, Deserialize, SerdeDiff)]
pub struct LockResponse {
    status: String,
    result: InnerLockResponse,
}

// TODO (howardwu): Add authentication.
#[post("/chunks/<chunk_id>/lock", data = "<participant_id>")]
pub fn lock_post(
    coordinator: State<Coordinator>,
    chunk_id: u64,
    participant_id: String,
) -> Result<Json<LockResponse>, Status> {
    match coordinator.lock_chunk(chunk_id, participant_id) {
        Ok(_) => Ok(Json(LockResponse {
            status: format!("ok"),
            result: InnerLockResponse { chunk_id, locked: true },
        })),
        Err(CoordinatorError::UnauthorizedChunkContributor) => Err(Status::Unauthorized),
        Err(CoordinatorError::LockAlreadyAcquired) => Err(Status::Unauthorized),
        Err(CoordinatorError::MissingChunk) => Err(Status::PreconditionFailed),
        Err(CoordinatorError::FailedToUpdateChunk) => Err(Status::InternalServerError),
        _ => Err(Status::NotFound),
    }

    // if !round.is_authorized_contributor(participant_id.clone()) {
    //     error!("Not authorized for /chunks/{}/lock", chunk_id);
    //     return Err(Status::Unauthorized);
    // }
    //
    // // TODO (howardwu): Account for the case where the participant is already the lock holder.
    // let is_locked = round.try_lock_chunk(chunk_id, participant_id);
}

#[cfg(test)]
mod test {
    use super::LockResponse;
    use crate::testing::prelude::*;

    #[test]
    #[serial]
    fn test_lock_post() {
        let client = test_client().unwrap();

        let mut response = client.post("/chunks/0/lock").body(TEST_CONTRIBUTOR_ID_1).dispatch();
        let response_body = response.body_string();
        assert_eq!(Status::Ok, response.status());
        assert_eq!(Some(ContentType::JSON), response.content_type());
        assert!(response_body.is_some());

        let candidate: LockResponse = serde_json::from_str(&response_body.unwrap()).unwrap();
        assert_eq!("ok", &candidate.status);
        assert_eq!(0, candidate.result.chunk_id);
        assert_eq!(true, candidate.result.locked);
    }
}
