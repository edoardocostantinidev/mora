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
    fn create_container(&mut self, container_id: Self::ContainerId) -> MoraResult<()>;

    /// Deletes a container.
    /// This will delete all items in the container irreversibly.
    fn delete_container(&mut self, container_id: Self::ContainerId) -> MoraResult<()>;

    /// Lists all containers.
    fn list_containers(&self) -> MoraResult<Vec<Self::ContainerId>>;

    /// Gets an item by its sort key.
    /// Returns `None` if the item does not exist.
    fn get_item(
        &self,
        container_id: Self::ContainerId,
        sort_key: Self::SortKey,
    ) -> MoraResult<Option<Self::Item>>;

    /// Gets a range of items by their sort key (inclusive).
    fn get_items_range(
        &self,
        container_id: Self::ContainerId,
        start_key: Self::SortKey,
        end_key: Self::SortKey,
    ) -> MoraResult<Vec<Self::Item>>;

    /// Gets the first n items by their sort key.
    fn get_n_items(&self, container_id: Self::ContainerId, n: usize)
        -> MoraResult<Vec<Self::Item>>;

    /// Deletes an item by its sort key.
    fn delete_item(
        &mut self,
        container_id: Self::ContainerId,
        sort_key: Self::SortKey,
    ) -> MoraResult<()>;

    /// Deletes a range of items by their sort key (inclusive).
    fn delete_items_range(
        &mut self,
        container_id: Self::ContainerId,
        start_key: Self::SortKey,
        end_key: Self::SortKey,
    ) -> MoraResult<()>;

    /// Stores an item by its sort key.
    /// If the item already exists, it will be overwritten.
    fn store_item(
        &mut self,
        container_id: Self::ContainerId,
        sort_key: Self::SortKey,
        item: Self::Item,
    ) -> MoraResult<()>;
}
