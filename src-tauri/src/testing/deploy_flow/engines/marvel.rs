use crate::testing::deploy_flow::{run_engine, EngineKey, FlowReport};

pub fn run() -> Result<(), FlowReport> {
    run_engine(EngineKey::Marvel)
}
