pub(crate) trait Selectable {
    fn is_selected(&self) -> bool;
    fn set_selected(&mut self, selected: bool);
}
