use std::{mem::MaybeUninit, sync::Once};

use crate::{database::KvStore, error::KvStoreError};

static mut KVSTORE: MaybeUninit<KvStore> = MaybeUninit::uninit();
static INIT: Once = Once::new();

impl KvStore {
    pub fn init(self) {
        unsafe {
            INIT.call_once(|| {
                KVSTORE.write(self);
            });
        }
    }
}

pub fn kvstore() -> Result<&'static KvStore, KvStoreError> {
    match INIT.is_completed() {
        true => unsafe { Ok(KVSTORE.assume_init_ref()) },
        false => Err(KvStoreError::Initialize),
    }
}
