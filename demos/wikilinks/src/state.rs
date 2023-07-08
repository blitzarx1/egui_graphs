#[derive(Default)]
pub enum State {
    #[default]
    Input,
    InputError,
    LoadingFirstLink,
    LoadingError,
    GraphAndLoading,
    Graph,
}
