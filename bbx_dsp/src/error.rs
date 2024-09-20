pub type Result<T> = std::result::Result<T, BbxAudioDspError>;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum BbxAudioDspError {
    #[error("block (`{0}`) has no inputs")]
    BlockHasNoInputs(String),

    #[error("block (`{0}`) has no outputs")]
    BlockHasNoOutputs(String),

    #[error("cannot add effector block to graph")]
    CannotAddEffectorBlock,

    #[error("cannot add generator block to graph")]
    CannotAddGeneratorBlock,

    #[error("cannot retrieve the current block (`{0}`)")]
    CannotRetrieveCurrentBlock(String),

    #[error("cannot retrieve the destination block (`{0}`)")]
    CannotRetrieveDestinationBlock(String),

    #[error("cannot retrieve the source block (`{0}`)")]
    CannotRetrieveSourceBlock(String),

    #[error("cannot update the graph's processing order")]
    CannotUpdateGraphProcessingOrder,

    #[error("connection has already been created")]
    ConnectionAlreadyCreated,

    #[error("connection has no corresponding block")]
    ConnectionHasNoBlock,

    #[error("graph contains a cycle (detected on block `{0}`)")]
    GraphContainsCycle(String),

    #[error("unknown error")]
    Unknown,
}
