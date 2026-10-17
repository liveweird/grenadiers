/// An upgrade concealed beneath a soft block until that block is destroyed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PickupKind {
    BombCapacity,
    BlastRange,
    MovementSpeed,
}
