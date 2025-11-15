use std::collections::HashMap;

use crate::result::MoraResult;

pub trait Storage {
    type ContainerId;
    type SortKey;
    type Item;

    /// Loads the storage engine.
    fn load() -> MoraResult<Self>
    where
        Self: Sized;

    /// Creates a new container.
    /// Each container is capable of storing an ordered set of items.
    /// The items are ordered by their sort key.
    fn create_container(&mut self, container_id: &Self::ContainerId) -> MoraResult<()>;

    /// Deletes a container.
    /// This will delete all items in the container irreversibly.
    fn delete_container(&mut self, container_id: &Self::ContainerId) -> MoraResult<()>;

    /// Lists all containers.
    fn list_containers(&self) -> MoraResult<Vec<Self::ContainerId>>;

    /// Deletes an item by its sort key.
    fn delete_item(
        &mut self,
        container_id: &Self::ContainerId,
        item_sort_key: &Self::SortKey,
    ) -> MoraResult<()>;

    /// Stores an item by its sort key.
    /// If the item already exists, it will be overwritten.
    fn store_item(
        &mut self,
        container_id: &Self::ContainerId,
        item_sort_key: &Self::SortKey,
        item: &Self::Item,
    ) -> MoraResult<()>;

    /// Stores multiple items in a single batch operation.
    /// This is more efficient than calling store_item multiple times.
    fn store_items(
        &mut self,
        container_id: &Self::ContainerId,
        items: &[(Self::SortKey, Self::Item)],
    ) -> MoraResult<()>
    where
        Self::SortKey: Clone,
        Self::Item: Clone;

    /// Get all items in a container.
    fn get_all_items(
        &mut self,
        container_id: &Self::ContainerId,
    ) -> MoraResult<HashMap<Self::SortKey, Self::Item>>;

    /// Deletes multiple items by their sort keys.
    fn delete_items(
        &mut self,
        container_id: &Self::ContainerId,
        item_sort_keys: &[Self::SortKey],
    ) -> MoraResult<()>;
}
