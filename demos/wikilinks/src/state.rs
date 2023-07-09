use log::info;

/// The state of the application
#[derive(Default, PartialEq, Debug)]
pub enum State {
    /// The user is entering an article url
    #[default]
    Input,
    /// The url was invalid
    InputError,
    ///Drawing graph, loading links
    GraphAndLoading,
    /// Error while loading links with some links loaded
    GraphAndLoadingError,
    /// All links were loaded now we have only the graph
    Graph,
}

#[derive(Debug)]
pub enum Fork {
    Success,
    Failure,
}

pub fn next(state: &State, fork: Fork) -> State {
    let to = match state {
        State::Input => match fork {
            Fork::Success => State::GraphAndLoading,
            Fork::Failure => State::InputError,
        },
        State::InputError => match fork {
            Fork::Success => State::Input,
            Fork::Failure => State::InputError,
        },
        State::GraphAndLoading => match fork {
            Fork::Success => State::Graph,
            Fork::Failure => State::GraphAndLoadingError,
        },
        State::GraphAndLoadingError => match fork {
            Fork::Success => State::Graph,
            Fork::Failure => State::GraphAndLoadingError,
        },
        State::Graph => match fork {
            Fork::Success => State::Graph,
            Fork::Failure => State::Graph,
        },
    };

    info!(
        "changed app state {:?} -> {:?} with fork {:?}",
        state, to, fork
    );

    to
}
