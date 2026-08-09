use crate::infra::Monolith;
use crate::infra::seeders::Seeder;

pub fn name() -> &'static str {
    "0001_google_fonts"
}

pub async fn run(mono: &Monolith) -> Result<(), anyhow::Error> {
    Ok(())
}
