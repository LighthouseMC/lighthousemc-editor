use std::ops::Deref;


#[derive(Clone)]
pub(crate) struct Dirty<T> {
    value : T,
    dirty : bool
}

impl<T> Dirty<T> {

    pub const fn new_dirty(value : T) -> Self {
        Self { value, dirty : true }
    }

    pub const fn new_clean(value : T) -> Self {
        Self { value, dirty : false }
    }

}

impl<T> Dirty<T> {

    pub fn mark_dirty(dirty : &mut Self) {
        dirty.dirty = true;
    }

    pub fn take_dirty(dirty : &mut Self) -> bool {
        let was_dirty = dirty.dirty;
        dirty.dirty = false;
        was_dirty
    }

}

impl<T : PartialEq> Dirty<T> {

    pub fn set(dirty : &mut Self, value : T) {
        if (dirty.value != value) {
            dirty.value = value;
            dirty.dirty = true;
        }
    }

    pub fn set_silent(dirty : &mut Self, value : T) {
        dirty.value = value;
    }

}

impl<T> Deref for Dirty<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
