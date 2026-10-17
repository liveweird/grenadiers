use std::{sync::Arc, time::Instant as StdInstant};

use crate::game::sessions::ConnectionId;

use super::*;
use crate::game::rooms::{Registry, RoomHandle};

fn rooms() -> Arc<GameRooms> {
    GameRooms::with_random_fill(StdInstant::now(), |bytes| {
        bytes.fill(0);
        Ok(())
    })
}

fn binding(generation: u64) -> BindingKey {
    BindingKey {
        connection_id: ConnectionId(1),
        generation,
    }
}

fn register_owner(rooms: &Arc<GameRooms>) -> (RoomHandle, PreparedOwner, RoomIdentity) {
    let mut registry = rooms.lock();
    register_owner_in(rooms, &mut registry)
}

fn register_owner_in(
    rooms: &Arc<GameRooms>,
    registry: &mut Registry,
) -> (RoomHandle, PreparedOwner, RoomIdentity) {
    let identity = rooms.generate_identity(registry).unwrap();
    let (commands, owner) = PreparedOwner::new(identity.clone(), Instant::now());
    let handle = RoomHandle { commands };
    registry
        .codes
        .insert(identity.code.clone(), identity.id.clone());
    registry.rooms.insert(identity.id.clone(), handle.clone());
    (handle, owner, identity)
}

fn assert_unregistered(rooms: &GameRooms, identity: &RoomIdentity) {
    let registry = rooms.lock();
    assert!(!registry.rooms.contains_key(&identity.id));
    assert!(!registry.codes.contains_key(&identity.code));
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn abort_before_first_poll_releases_registration() {
    let rooms = rooms();
    let (_handle, owner, identity) = register_owner(&rooms);

    let task = owner.start(Arc::downgrade(&rooms));
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());

    assert_unregistered(&rooms, &identity);
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn abort_after_admission_releases_registration_when_admission_drops() {
    let rooms = rooms();
    let (handle, owner, identity) = register_owner(&rooms);
    let task = owner.start(Arc::downgrade(&rooms));
    let admission = rooms.admit(handle, binding(1)).await.unwrap();

    task.abort();
    drop(admission);
    assert!(task.await.unwrap_err().is_cancelled());

    assert_unregistered(&rooms, &identity);
}
