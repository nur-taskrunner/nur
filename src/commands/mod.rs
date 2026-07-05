mod nur;

use nu_protocol::engine::{EngineState, StateWorkingSet};
pub(crate) use nur::Nur;

pub(crate) fn create_nu_context(engine_state: EngineState) -> EngineState {
    nu_cli::add_cli_context(engine_state)
}

pub(crate) fn create_nur_context(mut engine_state: EngineState) -> EngineState {
    // Add nur own commands
    let delta = {
        let mut working_set = StateWorkingSet::new(&engine_state);
        working_set.add_decl(Box::new(nur::Nur));
        working_set.render()
    };

    if let Err(err) = engine_state.merge_delta(delta) {
        eprintln!("Error creating nur command context: {err:?}");
    }

    engine_state
}
