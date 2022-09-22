use std::io::stdout;
use tokio::task::JoinError;
use zero2prod::{
    configuration::get_configuration,
    issue_delivery_worker::run_worker_until_stopped,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let configuration = get_configuration().unwrap();

    let subscriber = get_subscriber("zero2prod".into(), "info".into(), stdout);

    init_subscriber(subscriber);

    let application = Application::build(configuration.clone()).await?;
    let application_task = tokio::spawn(application.run_until_stopped());
    let worker_task = tokio::spawn(run_worker_until_stopped(configuration));

    tokio::select! {
        o = application_task => report_exit("API", o),
        o = worker_task => report_exit("Worker", o),
    };

    Ok(())
}

fn report_exit(
    task_name: &str,
    outcome: Result<Result<(), impl std::fmt::Debug + std::fmt::Display>, JoinError>,
) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("'{}', has exited", task_name)
        },
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "'{}' failed.",
                task_name
            )
        },
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "'{}' task failed to complete.",
                task_name
            )
        }
    }
}
