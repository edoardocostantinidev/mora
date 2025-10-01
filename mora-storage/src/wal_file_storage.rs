use mora_core::{result::MoraResult, traits::storage::Storage};

pub struct WalFileStorage;

impl Storage for WalFileStorage {
    type ContainerId = String;

    type SortKey = String;

    type Item = Vec<u8>;

    fn create_container(container_id: Self::ContainerId) -> MoraResult<()> {
        todo!()
    }

    fn delete_container(container_id: Self::ContainerId) -> MoraResult<()> {
        todo!()
    }

    fn list_containers() -> MoraResult<Vec<Self::ContainerId>> {
        todo!()
    }

    fn get_item(
        container_id: Self::ContainerId,
        sort_key: Self::SortKey,
    ) -> MoraResult<Option<Self::Item>> {
        todo!()
    }

    fn get_items_range(
        container_id: Self::ContainerId,
        start_key: Self::SortKey,
        end_key: Self::SortKey,
    ) -> MoraResult<Vec<Self::Item>> {
        todo!()
    }

    fn get_n_items(container_id: Self::ContainerId, n: usize) -> MoraResult<Vec<Self::Item>> {
        todo!()
    }

    fn delete_item(container_id: Self::ContainerId, sort_key: Self::SortKey) -> MoraResult<()> {
        todo!()
    }

    fn delete_items_range(
        container_id: Self::ContainerId,
        start_key: Self::SortKey,
        end_key: Self::SortKey,
    ) -> MoraResult<()> {
        todo!()
    }

    fn store_item(
        container_id: Self::ContainerId,
        sort_key: Self::SortKey,
        item: Self::Item,
    ) -> MoraResult<()> {
        todo!()
    }
}
