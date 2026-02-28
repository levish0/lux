#[derive(Debug, Clone, Copy)]
pub(super) struct AccessMode {
    pub(super) is_read: bool,
    pub(super) is_write: bool,
}

pub(super) const READ: AccessMode = AccessMode {
    is_read: true,
    is_write: false,
};

pub(super) const WRITE: AccessMode = AccessMode {
    is_read: false,
    is_write: true,
};

pub(super) const READ_WRITE: AccessMode = AccessMode {
    is_read: true,
    is_write: true,
};
