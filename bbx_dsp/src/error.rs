pub type Result<T> = std::result::Result<T, BbxAudioDspError>;

/// The error type for the `bbx_dsp` crate.
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum BbxAudioDspError {
    #[error("cannot add node to graph")]
    CannotAddNode,

    #[error("cannot add effector node to graph")]
    CannotAddEffectorNode,

    #[error("cannot add generator node to graph")]
    CannotAddGeneratorNode,

    #[error("cannot add modulator node to graph")]
    CannotAddModulatorNode,

    #[error("cannot retrieve the current node (`{0}`)")]
    CannotRetrieveCurrentNode(String),

    #[error("cannot retrieve the destination node (`{0}`)")]
    CannotRetrieveDestinationNode(String),

    #[error("cannot retrieve the source node (`{0}`)")]
    CannotRetrieveSourceNode(String),

    #[error("cannot update the graph's processing order")]
    CannotUpdateGraphProcessingOrder,

    #[error("connection has already been created")]
    ConnectionAlreadyCreated,

    #[error("connection has no corresponding node")]
    ConnectionHasNoNode,

    #[error("graph contains a cycle (detected on node `{0}`)")]
    GraphContainsCycle(String),

    #[error("graph has non-converging paths")]
    GraphContainsNonConvergingPaths,

    #[error("modulation has already been created")]
    ModulationAlreadyCreated,

    #[error("node (`{0}`) has no inputs")]
    NodeHasNoInputs(String),

    #[error("node (`{0}`) has no outputs")]
    NodeHasNoOutputs(String),

    #[error("unknown error")]
    Unknown,
}
