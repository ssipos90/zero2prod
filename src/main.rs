use std::io::stdout;
use zero2prod::{
    configuration::get_configuration,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().unwrap();

    let subscriber = get_subscriber("zero2prod".into(), "info".into(), stdout);

    init_subscriber(subscriber);

    let application = Application::build(configuration).await?;

    application.run_until_stopped().await?;

    Ok(())
}
