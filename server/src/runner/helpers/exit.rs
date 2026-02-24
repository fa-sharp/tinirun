use bollard::models::ContainerWaitResponse;
use tokio::time::error::Elapsed;

/// Returns a tuple containing a boolean indicating whether the exit was due to a timeout,
/// and the exit code of the program if available.
pub fn process_exit_status(
    exit_result: Result<Option<Result<ContainerWaitResponse, bollard::errors::Error>>, Elapsed>,
) -> (bool, Option<i64>) {
    let is_timeout = exit_result.is_err();
    let exit_code = match exit_result {
        Ok(Some(Ok(res))) => Some(res.status_code),
        Ok(Some(Err(err))) => match err {
            bollard::errors::Error::DockerContainerWaitError { code, .. } => Some(code),
            _ => None,
        },
        _ => None,
    };

    (is_timeout, exit_code)
}
